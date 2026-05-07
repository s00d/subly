use super::{SyncConfig, SyncPayload, SyncProviderType};
use serde::{Deserialize, Serialize};
mod gdrive;
mod dropbox;
mod onedrive;
mod webdav;
pub(crate) mod icloud;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) mod icloud_native;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub r#type: SyncProviderType,
    pub name: String,
    pub icon: String,
    pub fields: Vec<ProviderField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderField {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub secret: bool,
    pub placeholder: Option<String>,
    pub input_type: Option<String>,
    pub help_text: Option<String>,
    pub validation: Option<ProviderFieldValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderFieldValidation {
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
}

pub fn providers_list() -> Vec<ProviderInfo> {
    vec![
        icloud::descriptor(),
        gdrive::descriptor(),
        dropbox::descriptor(),
        onedrive::descriptor(),
        webdav::descriptor(),
    ]
}

pub fn provider_auth_url(provider: &SyncProviderType, cfg: &SyncConfig) -> Option<String> {
    match provider {
        SyncProviderType::Gdrive => gdrive::auth_url(),
        SyncProviderType::Dropbox => dropbox::auth_url(cfg),
        SyncProviderType::Onedrive => onedrive::auth_url(cfg),
        _ => None,
    }
}

pub async fn provider_download(
    app: &tauri::AppHandle,
    provider: &SyncProviderType,
    cfg: &SyncConfig,
    access_token: Option<&str>,
) -> Result<Option<SyncPayload>, String> {
    match provider {
        SyncProviderType::Gdrive => gdrive::download(cfg, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Dropbox => dropbox::download(cfg, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Onedrive => onedrive::download(cfg, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Webdav => webdav::download(cfg).await,
        SyncProviderType::Icloud => icloud::download(app, cfg).await,
    }
}

pub async fn provider_upload(
    app: &tauri::AppHandle,
    provider: &SyncProviderType,
    cfg: &SyncConfig,
    payload: &SyncPayload,
    access_token: Option<&str>,
) -> Result<(), String> {
    match provider {
        SyncProviderType::Gdrive => gdrive::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Dropbox => dropbox::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Onedrive => onedrive::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Webdav => webdav::upload(cfg, payload).await,
        SyncProviderType::Icloud => icloud::upload(app, cfg, payload).await,
    }
}
