use super::super::{decode_sync_payload, encode_sync_payload, oauth, SyncConfig, SyncPayload, SYNC_FILENAME};

#[derive(Debug, serde::Deserialize)]
struct GdriveFileRef {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct GdriveFilesListResponse {
    #[serde(default)]
    files: Vec<GdriveFileRef>,
}

pub fn descriptor() -> super::ProviderInfo {
    super::ProviderInfo {
        r#type: super::SyncProviderType::Gdrive,
        name: "Google Drive".to_string(),
        icon: "gdrive".to_string(),
        fields: vec![],
        auth_kind: super::SyncAuthKind::Oauth,
        requires_save_before_connect: false,
    }
}

pub fn auth_url() -> Option<String> {
    oauth::google_authorize_url().ok()
}

pub(crate) fn gdrive_sync_message_key(err: &crate::errors::AppError) -> Option<&'static str> {
    let msg = err.to_string().to_lowercase();
    // Own errors only: keys are tried in a chain, and a bare "401" match here used to label
    // other providers' auth failures as this provider's.
    if !msg.contains("gdrive") {
        return None;
    }
    if msg.contains("401") || msg.contains("unauthorized") {
        return Some("sync_gdrive_unauthorized");
    }
    if msg.contains("gdrive") {
        return Some("sync_gdrive_error");
    }
    None
}

/// `files.list` for the sync file: skip trashed copies (they still match by name) and sort newest
/// first, since without `orderBy` Drive returns matches in arbitrary order and `.first()` could pick
/// a stale duplicate (two devices creating the file at once leave two with the same name).
fn sync_file_list_url() -> String {
    url::Url::parse_with_params(
        "https://www.googleapis.com/drive/v3/files",
        &[
            ("spaces", "appDataFolder"),
            ("q", &format!("name='{SYNC_FILENAME}' and trashed=false")),
            ("orderBy", "modifiedTime desc"),
            ("fields", "files(id)"),
        ],
    )
    .map(|u| u.to_string())
    .unwrap_or_default()
}

fn gdrive_http_err(context: &str, status: tauri_plugin_http::reqwest::StatusCode) -> crate::errors::AppError {
    crate::errors::AppError::from(format!("gdrive {} failed: {}", context, status))
}

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, crate::errors::AppError> {
    let client = super::http_client();
    let list_url = sync_file_list_url();
    let list_resp = client
        .get(list_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !list_resp.status().is_success() {
        return Err(gdrive_http_err("list", list_resp.status()));
    }
    let list_json: GdriveFilesListResponse = list_resp.json().await.map_err(|e| e.to_string())?;
    let file_id = list_json.files.first().map(|f| f.id.clone());
    let Some(file_id) = file_id else { return Ok(None) };
    let resp = client
        .get(format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        if super::download_status_means_missing(resp.status().as_u16()) {
            return Ok(None);
        }
        return Err(gdrive_http_err("download", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

pub async fn upload(_cfg: &SyncConfig, payload: &SyncPayload, access_token: &str) -> Result<(), crate::errors::AppError> {
    let client = super::http_client();
    let list_url = sync_file_list_url();
    let list_resp = client
        .get(list_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    // A failed list must not fall through to "create": that leaves two sync files with the same
    // name and later downloads may pick the stale one.
    if !list_resp.status().is_success() {
        return Err(gdrive_http_err("list", list_resp.status()));
    }
    let list_json: GdriveFilesListResponse = list_resp.json().await.map_err(|e| e.to_string())?;
    let file_id = list_json.files.first().map(|f| f.id.clone());
    let compressed = encode_sync_payload(payload)?;
    let boundary = format!("subly_boundary_{}", chrono::Utc::now().timestamp_millis());
    let metadata = if file_id.is_some() {
        serde_json::json!({"name": SYNC_FILENAME})
    } else {
        serde_json::json!({"name": SYNC_FILENAME, "parents": ["appDataFolder"]})
    };
    let metadata_vec = serde_json::to_vec(&metadata).map_err(|e| e.to_string())?;
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend_from_slice(&metadata_vec);
    body.extend_from_slice(format!("\r\n--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Type: application/zstd\r\n\r\n");
    body.extend_from_slice(&compressed);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let has_existing = file_id.is_some();
    let url = if let Some(id) = file_id {
        format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=multipart", id)
    } else {
        "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart".to_string()
    };
    let req = if has_existing { client.patch(url) } else { client.post(url) };
    let resp = req
        .bearer_auth(access_token)
        .header("Content-Type", format!("multipart/related; boundary={}", boundary))
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(gdrive_http_err("upload", resp.status()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn list_url_skips_trash_and_orders_newest_first() {
        let u = url::Url::parse(&super::sync_file_list_url()).unwrap();
        let q: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
        assert!(q["q"].ends_with("and trashed=false"));
        assert_eq!(q["orderBy"], "modifiedTime desc");
        assert_eq!(q["spaces"], "appDataFolder");
    }
}
