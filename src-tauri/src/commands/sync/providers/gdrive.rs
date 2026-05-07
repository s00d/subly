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
    }
}

pub fn auth_url() -> Option<String> {
    oauth::google_authorize_url().ok()
}

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, String> {
    let client = tauri_plugin_http::reqwest::Client::new();
    let list_url = format!(
        "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='{}'&fields=files(id)",
        SYNC_FILENAME
    );
    let list_resp = client
        .get(list_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !list_resp.status().is_success() {
        return Ok(None);
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
        return Ok(None);
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

pub async fn upload(_cfg: &SyncConfig, payload: &SyncPayload, access_token: &str) -> Result<(), String> {
    let client = tauri_plugin_http::reqwest::Client::new();
    let list_url = format!(
        "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='{}'&fields=files(id)",
        SYNC_FILENAME
    );
    let list_resp = client
        .get(list_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let file_id = if list_resp.status().is_success() {
        let list_json: GdriveFilesListResponse = list_resp.json().await.map_err(|e| e.to_string())?;
        list_json.files.first().map(|f| f.id.clone())
    } else {
        None
    };
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
        return Err(format!("gdrive upload failed: {}", resp.status()));
    }
    Ok(())
}
