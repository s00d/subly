//! OS credential storage via [`tauri_plugin_keyring_store::KeyringStore`] (0.2+).
//!
//! Service name must match the plugin default (`com.s00d.subly`). iOS writes use
//! `AfterFirstUnlock` inside the plugin.

use std::sync::OnceLock;

use tauri_plugin_keyring_store::{Error as KeyringError, KeyringStore};

use crate::errors::AppError;

/// Matches `identifier` in `tauri.conf.json` and the plugin default service.
const SERVICE: &str = "com.s00d.subly";

/// IPC / toast code when Keychain is locked (maps from [`KeyringError::KeychainLocked`]).
pub const ERR_KEYRING_LOCKED: &str = "keyring_locked";

static STORE: OnceLock<KeyringStore> = OnceLock::new();

fn store() -> &'static KeyringStore {
    STORE.get_or_init(|| KeyringStore::new(SERVICE))
}

fn map_err(e: KeyringError) -> AppError {
    match e {
        KeyringError::KeychainLocked => AppError::from(ERR_KEYRING_LOCKED),
        other => AppError::Message(other.to_string()),
    }
}

/// Foreground / user-initiated read.
pub fn get(account: &str) -> Result<Option<String>, AppError> {
    store().get_password(account).map_err(map_err)
}

/// Background-safe read: on iOS returns `None` when the device is locked; elsewhere same as [`get`].
pub fn get_for_background(account: &str) -> Result<Option<String>, AppError> {
    #[cfg(target_os = "ios")]
    {
        return store()
            .get_password_for_background(account)
            .map_err(map_err);
    }
    #[cfg(not(target_os = "ios"))]
    {
        get(account)
    }
}

/// Background-safe existence check for sync UI (iOS lock-aware).
pub fn exists_for_background(account: &str) -> Result<bool, AppError> {
    #[cfg(target_os = "ios")]
    {
        return store()
            .exists_nonempty_for_background(account)
            .map_err(map_err);
    }
    #[cfg(not(target_os = "ios"))]
    {
        Ok(store()
            .exists_nonempty(account)
            .map_err(map_err)?)
    }
}

pub fn set(account: &str, password: &str) -> Result<(), AppError> {
    store().set_password(account, password).map_err(map_err)
}

pub fn delete(account: &str) -> Result<(), AppError> {
    store().delete(account).map_err(map_err)
}
