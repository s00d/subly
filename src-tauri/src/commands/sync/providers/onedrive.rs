use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use super::super::{
    decode_sync_payload, encode_sync_payload, OAUTH_REDIRECT_URI, SyncConfig, SyncPayload, SYNC_FILENAME,
};

fn enc_query(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

/// Временное имя внутри `approot` для атомарной замены основного файла.
const ONEDRIVE_TMP_NAME: &str = "subly-sync-v1.upload.tmp";

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
            },
        ],
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

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, String> {
    let resp = tauri_plugin_http::reqwest::Client::new()
        .get(format!(
            "https://graph.microsoft.com/v1.0/me/drive/special/approot:/{}:/content",
            SYNC_FILENAME
        ))
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
    let raw = encode_sync_payload(payload)?;
    let put_url = format!(
        "https://graph.microsoft.com/v1.0/me/drive/special/approot:/{}:/content",
        ONEDRIVE_TMP_NAME
    );
    let put = client
        .put(&put_url)
        .bearer_auth(access_token)
        .header("Content-Type", "application/octet-stream")
        .body(raw)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !put.status().is_success() {
        return Err(format!("onedrive upload (tmp) failed: {}", put.status()));
    }

    let patch_url = format!(
        "https://graph.microsoft.com/v1.0/me/drive/special/approot:/{}:",
        ONEDRIVE_TMP_NAME
    );
    let patch = client
        .patch(&patch_url)
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "name": SYNC_FILENAME,
            "@microsoft.graph.conflictBehavior": "replace"
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !patch.status().is_success() {
        let _ = client
            .delete(patch_url)
            .bearer_auth(access_token)
            .send()
            .await;
        return Err(format!("onedrive rename (tmp→final) failed: {}", patch.status()));
    }

    Ok(())
}
