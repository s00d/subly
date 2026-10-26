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
                has_saved_value: None,
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
                has_saved_value: None,
            },
        ],
        auth_kind: super::SyncAuthKind::Oauth,
        requires_save_before_connect: true,
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

pub(crate) fn dropbox_sync_message_key(err: &crate::errors::AppError) -> Option<&'static str> {
    let msg = err.to_string().to_lowercase();
    // Own errors only: keys are tried in a chain, and a bare "401" match here used to label
    // other providers' auth failures as this provider's.
    if !msg.contains("dropbox") {
        return None;
    }
    if msg.contains("401") || msg.contains("unauthorized") {
        return Some("sync_oauth_required");
    }
    if msg.contains("dropbox") {
        return Some("sync_dropbox_error");
    }
    None
}

pub async fn download(_cfg: &SyncConfig, access_token: &str) -> Result<Option<SyncPayload>, crate::errors::AppError> {
    let resp = super::http_client()
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
        let status = resp.status();
        // Dropbox reports a missing file as 409 `path/not_found`, not 404.
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 409 && body.contains("not_found") {
            return Ok(None);
        }
        return Err(crate::errors::AppError::from(format!("dropbox download failed: {}", status)));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

fn dropbox_upload_arg(path: &str) -> String {
    serde_json::json!({
      "path": path,
      "mode": "overwrite",
      "autorename": false,
      "mute": true
    })
    .to_string()
}

pub async fn upload(_cfg: &SyncConfig, payload: &SyncPayload, access_token: &str) -> Result<(), crate::errors::AppError> {
    let client = super::http_client();
    let raw = encode_sync_payload(payload)?;
    // `files/upload` commits atomically, so write straight to the final path. The old tmp + `move_v2`
    // flow failed on every push after the first: `move_v2` cannot overwrite (`to/conflict`).
    let fin = dropbox_final_path();

    let put = client
        .post("https://content.dropboxapi.com/2/files/upload")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/octet-stream")
        .header(
            "Dropbox-API-Arg",
            dropbox_upload_arg(&fin),
        )
        .body(raw)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !put.status().is_success() {
        return Err(crate::errors::AppError::from(format!(
            "dropbox upload failed: {}",
            put.status()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn upload_targets_final_path_with_overwrite() {
        let arg: serde_json::Value = serde_json::from_str(&super::dropbox_upload_arg(&super::dropbox_final_path())).unwrap();
        assert_eq!(arg["path"], super::dropbox_final_path());
        assert_eq!(arg["mode"], "overwrite");
    }
}
