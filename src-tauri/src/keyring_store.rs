//! Persistent secret storage backed by the OS credential vault.
//!
//! On each supported platform a `keyring-core` 1.x backend crate is wired up
//! as the default store; the rest of the application uses the same
//! [`get`] / [`set`] / [`delete`] API regardless of platform.
//!
//! | Target     | Backend crate                            | Underlying store                          |
//! |------------|------------------------------------------|-------------------------------------------|
//! | macOS      | `apple-native-keyring-store` (keychain)  | Login keychain                            |
//! | iOS        | `apple-native-keyring-store` (protected) | Data Protection keychain                  |
//! | Windows    | `windows-native-keyring-store`           | Credential Manager                        |
//! | Linux      | `dbus-secret-service-keyring-store`      | Secret Service (gnome-keyring / KWallet)  |
//! | Android    | `android-native-keyring-store`           | Android Keystore + SharedPreferences      |
//!
//! The default store is registered exactly once on first use via
//! [`ensure_init`]. After that, every entry is created with
//! `keyring_core::Entry::new(SERVICE, account)`.

use std::sync::OnceLock;

use keyring_core::error::Error as KeyringError;
use keyring_core::Entry;

use crate::errors::AppError;

const SERVICE: &str = "com.s00d.subly";

/// Holds the result of the one-time backend registration. We cache it so
/// every subsequent call is a single atomic load.
static INIT: OnceLock<Result<(), String>> = OnceLock::new();

fn ensure_init() -> Result<(), AppError> {
    let stored = INIT.get_or_init(register_default_store);
    match stored {
        Ok(()) => Ok(()),
        Err(msg) => Err(AppError::Message(msg.clone())),
    }
}

#[cfg(target_os = "macos")]
fn register_default_store() -> Result<(), String> {
    let store = apple_native_keyring_store::keychain::Store::new()
        .map_err(|e| format!("apple keychain init failed: {e}"))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "ios")]
fn register_default_store() -> Result<(), String> {
    let store = apple_native_keyring_store::protected::Store::new()
        .map_err(|e| format!("apple protected store init failed: {e}"))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "windows")]
fn register_default_store() -> Result<(), String> {
    let store = windows_native_keyring_store::Store::new()
        .map_err(|e| format!("windows credential store init failed: {e}"))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "linux")]
fn register_default_store() -> Result<(), String> {
    let store = dbus_secret_service_keyring_store::Store::new()
        .map_err(|e| format!("dbus secret service init failed: {e}"))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(target_os = "android")]
fn register_default_store() -> Result<(), String> {
    let store = android_native_keyring_store::Store::new()
        .map_err(|e| format!("android keystore init failed: {e}"))?;
    keyring_core::set_default_store(store);
    Ok(())
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "windows",
    target_os = "linux",
    target_os = "android",
)))]
fn register_default_store() -> Result<(), String> {
    Err("no keyring backend compiled for this target".to_string())
}

fn entry(account: &str) -> Result<Entry, AppError> {
    ensure_init()?;
    Entry::new(SERVICE, account).map_err(|e| AppError::Message(e.to_string()))
}

pub fn get(account: &str) -> Result<Option<String>, AppError> {
    let entry = entry(account)?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Message(e.to_string())),
    }
}

/// Returns `true` when an entry for `account` exists and stores a non-empty
/// secret. Functionally equivalent to [`get`], but conveys intent ("is this
/// configured?") at the call site and discards the secret immediately.
pub fn exists(account: &str) -> Result<bool, AppError> {
    Ok(get(account)?.map(|v| !v.trim().is_empty()).unwrap_or(false))
}

pub fn set(account: &str, password: &str) -> Result<(), AppError> {
    let entry = entry(account)?;
    entry
        .set_password(password)
        .map_err(|e| AppError::Message(e.to_string()))
}

pub fn delete(account: &str) -> Result<(), AppError> {
    let entry = entry(account)?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Message(e.to_string())),
    }
}
