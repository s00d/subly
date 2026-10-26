use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use super::super::{
    decode_sync_payload, encode_sync_payload, OAUTH_REDIRECT_URI, SyncConfig, SyncPayload, SYNC_FILENAME,
};

fn enc_query(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

pub fn descriptor() -> super::ProviderInfo {
    super::ProviderInfo {
        r#type: super::SyncProviderType::Onedrive,
        name: "OneDrive".to_string(),
        icon: "onedrive".to_string(),
        fields: vec![
            super::ProviderField {
                key: "onedriveClientId".to_string(),
                label: "sync_field_onedrive_client_id".to_string(),
                required: true,
                secret: false,
                placeholder: Some("sync_placeholder_onedrive_client_id".to_string()),
                input_type: Some("text".to_string()),
                help_text: Some("sync_help_onedrive_client_id".to_string()),
                validation: Some(super::ProviderFieldValidation { min_length: Some(8), pattern: None }),
                has_saved_value: None,
            },
        ],
        auth_kind: super::SyncAuthKind::Oauth,
        requires_save_before_connect: true,
    }
}

pub fn auth_url(cfg: &SyncConfig) -> Option<String> {
    if cfg.onedrive_client_id.is_empty() {
        return None;
    }
    Some(format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state=onedrive",
        enc_query(&cfg.onedrive_client_id),
        enc_query(OAUTH_REDIRECT_URI),
        enc_query("Files.ReadWrite.AppFolder offline_access"),
    ))
}

pub(crate) fn onedrive_sync_message_key(err: &crate::errors::AppError) -> Option<&'static str> {
    let msg = err.to_string().to_lowercase();
    // Own errors only: keys are tried in a chain, and a bare "401" match here used to label
    // other providers' auth failures as this provider's.
    if !msg.contains("onedrive") {
        return None;
    }
    if msg.contains("401") || msg.contains("unauthorized") {
        return Some("sync_oauth_required");
    }
    if msg.contains("onedrive") {
        return Some("sync_onedrive_error");
    }
    None
}

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, crate::errors::AppError> {
    let resp = super::http_client()
        .get(sync_file_content_url())
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        if super::download_status_means_missing(resp.status().as_u16()) {
            return Ok(None);
        }
        return Err(crate::errors::AppError::from(format!("onedrive download failed: {}", resp.status())));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

/// Simple-upload URL of the sync file in the app folder (Graph allows up to 250 MB this way).
fn sync_file_content_url() -> String {
    format!(
        "https://graph.microsoft.com/v1.0/me/drive/special/approot:/{}:/content",
        SYNC_FILENAME
    )
}

pub async fn upload(_cfg: &SyncConfig, payload: &SyncPayload, access_token: &str) -> Result<(), crate::errors::AppError> {
    // One PUT to the final path: Graph replaces the content atomically and keeps version history.
    // The old tmp + PATCH rename relied on `@microsoft.graph.conflictBehavior` in a PATCH body,
    // which driveitem-update does not document (rename onto an existing name fails with 409).
    let raw = encode_sync_payload(payload)?;
    let put = super::http_client()
        .put(sync_file_content_url())
        .bearer_auth(access_token)
        .header("Content-Type", "application/octet-stream")
        .body(raw)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !put.status().is_success() {
        return Err(crate::errors::AppError::from(format!(
            "onedrive upload failed: {}",
            put.status()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn upload_and_download_share_the_final_path() {
        let url = super::sync_file_content_url();
        assert!(url.ends_with(&format!("approot:/{}:/content", super::SYNC_FILENAME)));
        assert!(!url.contains(".tmp"));
    }
}
