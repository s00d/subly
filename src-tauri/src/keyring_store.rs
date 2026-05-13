//! OS credential storage via [`tauri_plugin_keyring_store::KeyringStore`].
//! The service name must match the default in `tauri_plugin_keyring_store::init()`
//! (bundle identifier from `tauri.conf.json`).

use std::sync::OnceLock;

use tauri_plugin_keyring_store::KeyringStore;

use crate::errors::AppError;

/// Matches `identifier` in `tauri.conf.json` and the plugin default service.
const SERVICE: &str = "com.s00d.subly";

static STORE: OnceLock<KeyringStore> = OnceLock::new();

fn store() -> &'static KeyringStore {
    STORE.get_or_init(|| KeyringStore::new(SERVICE))
}

fn map_err(e: tauri_plugin_keyring_store::Error) -> AppError {
    AppError::Message(e.to_string())
}

pub fn get(account: &str) -> Result<Option<String>, AppError> {
    store().get_password(account).map_err(map_err)
}

/// Returns `true` when an entry exists with a non-empty secret.
pub fn exists(account: &str) -> Result<bool, AppError> {
    store().exists_nonempty(account).map_err(map_err)
}

pub fn set(account: &str, password: &str) -> Result<(), AppError> {
    store().set_password(account, password).map_err(map_err)
}

pub fn delete(account: &str) -> Result<(), AppError> {
    store().delete(account).map_err(map_err)
}
