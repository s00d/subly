use super::{generate_device_id, OAuthTokens, SyncConfig, SyncProviderType};

pub const SYNC_CONFIG_KEY: &str = "sync_config";
pub const GDRIVE_TOKEN_KEY: &str = "gdrive_tokens";
pub const DROPBOX_TOKEN_KEY: &str = "dropbox_tokens";
pub const ONEDRIVE_TOKEN_KEY: &str = "onedrive_tokens";

pub(crate) const KR_SYNC_DROPBOX_APP_SECRET: &str = "subly.sync.dropbox_app_secret";
pub(crate) const KR_SYNC_WEBDAV_PASSWORD: &str = "subly.sync.webdav_password";

/// Подпапка внутри ubiquity `Documents/` (iOS/macOS) или legacy `…/CloudDocs/<эта>/` на macOS.
pub const SYNC_ICLOUD_FOLDER: &str = "Subly-v1";
/// Имя файла синка (все провайдеры). Dropbox по-прежнему: `/Apps/Subly/<файл>`.
pub const SYNC_FILENAME: &str = "subly-sync-v2.sublysync";
pub const MAX_SYNC_PAYLOAD_BYTES: usize = 2_500_000;
/// Единый redirect для OAuth (Google / Dropbox / OneDrive). Должен совпадать с deep link в приложении.
pub const OAUTH_REDIRECT_URI: &str = "subly://oauth/callback";

/// Google OAuth client ID for the running platform (PKCE public client; no client secret in the app).
pub fn gdrive_oauth_client_id() -> &'static str {
    #[cfg(target_os = "ios")]
    {
        "237019351467-v1fu0a108r23g2kff6qllnl2p828d7hq.apps.googleusercontent.com"
    }
    #[cfg(target_os = "android")]
    {
        "237019351467-ankaki2h3esl0ml5pqodgvqtdgqcfrhp.apps.googleusercontent.com"
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        "237019351467-ankaki2h3esl0ml5pqodgvqtdgqcfrhp.apps.googleusercontent.com"
    }
}

fn parse_json_or_err<T: serde::de::DeserializeOwned>(key: &str, raw: &str) -> Result<T, String> {
    serde_json::from_str::<T>(raw).map_err(|e| format!("invalid {} payload: {}", key, e))
}

pub fn load_sync_config() -> Result<SyncConfig, String> {
    let raw = crate::redb_get_internal(SYNC_CONFIG_KEY.to_string())?;
    let mut cfg = match raw {
        Some(ref s) => parse_json_or_err(SYNC_CONFIG_KEY, s)?,
        None => SyncConfig::default(),
    };
    if cfg.device_id.is_empty() {
        cfg.device_id = generate_device_id();
        save_sync_config(&cfg)?;
    }
    Ok(cfg)
}

pub fn save_sync_config(cfg: &SyncConfig) -> Result<(), String> {
    let raw = serde_json::to_string(cfg).map_err(|e| e.to_string())?;
    crate::redb_set_internal(SYNC_CONFIG_KEY.to_string(), raw)
}

pub(crate) fn dropbox_app_secret() -> Result<String, String> {
    Ok(crate::keyring_store::get(KR_SYNC_DROPBOX_APP_SECRET)?.unwrap_or_default())
}

pub(crate) fn set_dropbox_app_secret(secret: &str) -> Result<(), String> {
    if secret.trim().is_empty() {
        crate::keyring_store::delete(KR_SYNC_DROPBOX_APP_SECRET)
    } else {
        crate::keyring_store::set(KR_SYNC_DROPBOX_APP_SECRET, secret)
    }
}

pub(crate) fn webdav_password() -> Result<String, String> {
    Ok(crate::keyring_store::get(KR_SYNC_WEBDAV_PASSWORD)?.unwrap_or_default())
}

pub(crate) fn set_webdav_password(password: &str) -> Result<(), String> {
    if password.trim().is_empty() {
        crate::keyring_store::delete(KR_SYNC_WEBDAV_PASSWORD)
    } else {
        crate::keyring_store::set(KR_SYNC_WEBDAV_PASSWORD, password)
    }
}

pub fn load_oauth_tokens(key: &str) -> Result<Option<OAuthTokens>, String> {
    let raw = crate::keyring_store::get(key)?;
    match raw {
        Some(ref s) => Ok(Some(parse_json_or_err(key, s)?)),
        None => Ok(None),
    }
}

pub fn save_oauth_tokens(key: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let raw = serde_json::to_string(tokens).map_err(|e| e.to_string())?;
    crate::keyring_store::set(key, &raw)
}

pub fn clear_oauth_tokens(key: &str) -> Result<(), String> {
    crate::keyring_store::delete(key)
}

pub fn token_key(provider: &SyncProviderType) -> Option<&'static str> {
    match provider {
        SyncProviderType::Gdrive => Some(GDRIVE_TOKEN_KEY),
        SyncProviderType::Dropbox => Some(DROPBOX_TOKEN_KEY),
        SyncProviderType::Onedrive => Some(ONEDRIVE_TOKEN_KEY),
        _ => None,
    }
}
