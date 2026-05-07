use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use super::super::{
    decode_sync_payload, encode_sync_payload, OAUTH_REDIRECT_URI, SyncConfig, SyncPayload, SYNC_FILENAME,
};

fn enc_query(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

fn dropbox_final_path() -> String {
    format!("/Apps/Subly/{}", SYNC_FILENAME)
}

fn dropbox_tmp_path() -> String {
    format!("/Apps/Subly/{}.tmp", SYNC_FILENAME)
}

pub fn descriptor() -> super::ProviderInfo {
    super::ProviderInfo {
        r#type: super::SyncProviderType::Dropbox,
        name: "Dropbox".to_string(),
        icon: "dropbox".to_string(),
        fields: vec![
            super::ProviderField {
                key: "dropboxAppKey".to_string(),
                label: "sync_field_dropbox_app_key".to_string(),
                required: true,
                secret: false,
                placeholder: Some("sync_placeholder_dropbox_app_key".to_string()),
                input_type: Some("text".to_string()),
                help_text: Some("sync_help_dropbox_app_key".to_string()),
                validation: Some(super::ProviderFieldValidation { min_length: Some(8), pattern: None }),
            },
            super::ProviderField {
                key: "dropboxAppSecret".to_string(),
                label: "sync_field_dropbox_app_secret".to_string(),
                required: true,
                secret: true,
                placeholder: Some("sync_placeholder_dropbox_app_secret".to_string()),
                input_type: Some("password".to_string()),
                help_text: Some("sync_help_dropbox_app_secret".to_string()),
                validation: Some(super::ProviderFieldValidation { min_length: Some(8), pattern: None }),
            },
        ],
    }
}

pub fn auth_url(cfg: &SyncConfig) -> Option<String> {
    if cfg.dropbox_app_key.is_empty() {
        return None;
    }
    Some(format!(
        "https://www.dropbox.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&token_access_type=offline&state=dropbox",
        enc_query(&cfg.dropbox_app_key),
        enc_query(OAUTH_REDIRECT_URI),
    ))
}

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, String> {
    let resp = tauri_plugin_http::reqwest::Client::new()
        .post("https://content.dropboxapi.com/2/files/download")
        .header("Authorization", format!("Bearer {}", access_token))
        .header(
            "Dropbox-API-Arg",
            serde_json::json!({ "path": dropbox_final_path() }).to_string(),
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

async fn dropbox_move_v2(
    client: &tauri_plugin_http::reqwest::Client,
    access_token: &str,
    from_path: &str,
    to_path: &str,
) -> Result<(), String> {
    let resp = client
        .post("https://api.dropboxapi.com/2/files/move_v2")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "from_path": from_path,
            "to_path": to_path,
            "autorename": false,
            "allow_shared_folder": false,
            "allow_ownership_transfer": false,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("dropbox move_v2 failed: {}", resp.status()));
    }
    Ok(())
}

pub async fn upload(_cfg: &SyncConfig, payload: &SyncPayload, access_token: &str) -> Result<(), String> {
    let client = tauri_plugin_http::reqwest::Client::new();
    let raw = encode_sync_payload(payload)?;
    let tmp = dropbox_tmp_path();
    let fin = dropbox_final_path();

    let put = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/octet-stream")
        .header(
            "Dropbox-API-Arg",
            serde_json::json!({
              "path": tmp,
              "mode": "overwrite",
              "autorename": false,
              "mute": true
            })
            .to_string(),
        )
        .body(raw)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !put.status().is_success() {
        return Err(format!("dropbox upload failed: {}", put.status()));
    }

    if let Err(e) = dropbox_move_v2(&client, access_token, &tmp, &fin).await {
        let _ = client
            .post("https://api.dropboxapi.com/2/files/delete_v2")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "path": tmp }))
            .send()
            .await;
        return Err(e);
    }

    Ok(())
}
