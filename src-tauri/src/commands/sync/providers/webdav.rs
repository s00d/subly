use tauri_plugin_http::reqwest::Method;
use super::super::{decode_sync_payload, encode_sync_payload, webdav_password, SyncConfig, SyncPayload, SYNC_FILENAME};

fn tmp_filename() -> String {
    format!("{}.tmp", SYNC_FILENAME)
}

pub fn descriptor() -> super::ProviderInfo {
    super::ProviderInfo {
        r#type: super::SyncProviderType::Webdav,
        name: "WebDAV".to_string(),
        icon: "webdav".to_string(),
        fields: vec![
            super::ProviderField {
                key: "webdavUrl".to_string(),
                label: "sync_field_webdav_url".to_string(),
                required: true,
                secret: false,
                placeholder: Some("sync_placeholder_webdav_url".to_string()),
                input_type: Some("url".to_string()),
                help_text: Some("sync_help_webdav_url".to_string()),
                validation: Some(super::ProviderFieldValidation { min_length: Some(8), pattern: Some(r"^https?://".to_string()) }),
                has_saved_value: None,
            },
            super::ProviderField {
                key: "webdavUsername".to_string(),
                label: "sync_field_webdav_username".to_string(),
                required: true,
                secret: false,
                placeholder: Some("sync_placeholder_webdav_username".to_string()),
                input_type: Some("text".to_string()),
                help_text: Some("sync_help_webdav_username".to_string()),
                validation: Some(super::ProviderFieldValidation { min_length: Some(1), pattern: None }),
                has_saved_value: None,
            },
            super::ProviderField {
                key: "webdavPassword".to_string(),
                label: "sync_field_webdav_password".to_string(),
                required: false,
                secret: true,
                placeholder: Some("sync_placeholder_webdav_password".to_string()),
                input_type: Some("password".to_string()),
                help_text: Some("sync_help_webdav_password".to_string()),
                validation: None,
                has_saved_value: None,
            },
        ],
    }
}

fn item_url(base: &str, name: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), name)
}

pub async fn download(cfg: &SyncConfig) -> Result<Option<SyncPayload>, crate::errors::AppError> {
    if cfg.webdav_url.is_empty() || cfg.webdav_username.is_empty() {
        return Ok(None);
    }
    let pwd = webdav_password()?;
    let url = item_url(&cfg.webdav_url, SYNC_FILENAME);
    let resp = tauri_plugin_http::reqwest::Client::new()
        .get(url)
        .basic_auth(cfg.webdav_username.clone(), Some(pwd))
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    Ok(Some(decode_sync_payload(&bytes)?))
}

pub async fn upload(cfg: &SyncConfig, payload: &SyncPayload) -> Result<(), crate::errors::AppError> {
    if cfg.webdav_url.is_empty() || cfg.webdav_username.is_empty() {
        return Err(crate::errors::AppError::from("webdav credentials missing"));
    }
    let pwd = webdav_password()?;
    let base = cfg.webdav_url.trim_end_matches('/').to_string();
    let tmp_name = tmp_filename();
    let tmp_url = item_url(&base, &tmp_name);
    let final_url = item_url(&base, SYNC_FILENAME);
    let raw = encode_sync_payload(payload)?;
    let raw_clone = raw.clone();
    let client = tauri_plugin_http::reqwest::Client::new();
    let auth = (cfg.webdav_username.clone(), Some(pwd));

    let put = client
        .put(&tmp_url)
        .basic_auth(auth.0.clone(), auth.1.clone())
        .header("Content-Type", "application/octet-stream")
        .body(raw_clone)
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    if !put.status().is_success() {
        return Err(crate::errors::AppError::from(format!(
            "webdav upload (.tmp) failed: {}",
            put.status()
        )));
    }

    let move_method =
        Method::from_bytes(b"MOVE").map_err(|_| crate::errors::AppError::from("MOVE method"))?;
    let mov = client
        .request(move_method, &tmp_url)
        .basic_auth(auth.0.clone(), auth.1.clone())
        .header("Destination", &final_url)
        .header("Overwrite", "T")
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;

    if mov.status().is_success() {
        return Ok(());
    }

    let del = client
        .delete(&final_url)
        .basic_auth(auth.0.clone(), auth.1.clone())
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    if !del.status().is_success() && del.status().as_u16() != 404 {
        let _ = client.delete(&tmp_url).basic_auth(auth.0.clone(), auth.1.clone()).send().await;
        return Err(crate::errors::AppError::from(format!(
            "webdav fallback delete final failed: {}",
            del.status()
        )));
    }

    let re_put = client
        .put(&final_url)
        .basic_auth(auth.0.clone(), auth.1.clone())
        .header("Content-Type", "application/octet-stream")
        .body(raw)
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    if !re_put.status().is_success() {
        return Err(crate::errors::AppError::from(format!(
            "webdav fallback put final failed: {}",
            re_put.status()
        )));
    }
    let _ = client.delete(&tmp_url).basic_auth(auth.0, auth.1).send().await;
    Ok(())
}
