use super::{SyncAuthKind, SyncConfig, SyncPayload, SyncProviderType};
use crate::keyring_store;
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
    pub auth_kind: SyncAuthKind,
    pub requires_save_before_connect: bool,
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
    /// `Some(true)` ⇔ the corresponding secret is already stored in the OS
    /// keyring. The field name itself never carries the value; this flag is
    /// only meant to drive UI affordances (mask, "Save" button enablement).
    /// `None` for non-secret fields.
    pub has_saved_value: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderFieldValidation {
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
}

/// Maps a `ProviderField.key` (the camelCase identifier shipped to the UI) to
/// the keyring account name under which we persist its secret. The lookup is
/// deliberately exhaustive and falls back to `None` for fields we do *not*
/// store in the keyring — those should always render with an empty input.
fn keyring_account_for_field(field_key: &str) -> Option<&'static str> {
    match field_key {
        "dropboxAppSecret" => Some(super::config::KR_SYNC_DROPBOX_APP_SECRET),
        "webdavPassword" => Some(super::config::KR_SYNC_WEBDAV_PASSWORD),
        _ => None,
    }
}

/// Fills `has_saved_value` for every secret field by probing the keyring.
/// Errors from the keyring (e.g. locked iOS data-protection class) collapse
/// to `Some(false)` — the UI then prompts the user to type a fresh value,
/// which is the correct fallback for an unreachable secret.
fn annotate_saved_secrets(mut info: ProviderInfo) -> ProviderInfo {
    for field in info.fields.iter_mut() {
        if !field.secret {
            field.has_saved_value = None;
            continue;
        }
        let saved = keyring_account_for_field(&field.key)
            .map(|account| keyring_store::exists_for_background(account).unwrap_or(false))
            .unwrap_or(false);
        field.has_saved_value = Some(saved);
    }
    info
}

pub fn providers_list() -> Vec<ProviderInfo> {
    vec![
        icloud::descriptor(),
        gdrive::descriptor(),
        dropbox::descriptor(),
        onedrive::descriptor(),
        webdav::descriptor(),
    ]
    .into_iter()
    .map(annotate_saved_secrets)
    .collect()
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
) -> Result<Option<SyncPayload>, crate::errors::AppError> {
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
) -> Result<(), crate::errors::AppError> {
    match provider {
        SyncProviderType::Gdrive => gdrive::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Dropbox => dropbox::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Onedrive => onedrive::upload(cfg, payload, access_token.ok_or("missing access token")?).await,
        SyncProviderType::Webdav => webdav::upload(cfg, payload).await,
        SyncProviderType::Icloud => icloud::upload(app, cfg, payload).await,
    }
}

pub(crate) fn provider_io_message_key(err: &crate::errors::AppError) -> Option<&'static str> {
    icloud::icloud_sync_message_key(err)
        .or_else(|| gdrive::gdrive_sync_message_key(err))
        .or_else(|| dropbox::dropbox_sync_message_key(err))
        .or_else(|| onedrive::onedrive_sync_message_key(err))
        .or_else(|| webdav::webdav_sync_message_key(err))
}

pub async fn webdav_test_connection(cfg: &SyncConfig) -> Result<(), crate::errors::AppError> {
    webdav::test_connection(cfg).await
}
