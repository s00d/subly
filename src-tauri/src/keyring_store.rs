//! Secrets in the OS credential store (Keychain / Credential Manager / Secret Service / Keystore).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const SERVICE: &str = "com.s00d.subly";
static SHADOW_SECRETS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn shadow_secrets() -> &'static Mutex<HashMap<String, String>> {
    SHADOW_SECRETS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get(account: &str) -> Result<Option<String>, crate::errors::AppError> {
    let entry = keyring::Entry::new(SERVICE, account).map_err(|e| {
        crate::errors::AppError::Message(e.to_string())
    })?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => {
            let guard = shadow_secrets()
                .lock()
                .map_err(|_| crate::errors::AppError::from("shadow keyring cache lock poisoned"))?;
            Ok(guard.get(account).cloned())
        }
        Err(e) => Err(crate::errors::AppError::Message(e.to_string())),
    }
}

pub fn set(account: &str, password: &str) -> Result<(), crate::errors::AppError> {
    let entry = keyring::Entry::new(SERVICE, account).map_err(|e| {
        crate::errors::AppError::Message(e.to_string())
    })?;
    entry
        .set_password(password)
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let mut guard = shadow_secrets()
        .lock()
        .map_err(|_| crate::errors::AppError::from("shadow keyring cache lock poisoned"))?;
    guard.insert(account.to_string(), password.to_string());
    Ok(())
}

pub fn delete(account: &str) -> Result<(), crate::errors::AppError> {
    let entry = keyring::Entry::new(SERVICE, account).map_err(|e| {
        crate::errors::AppError::Message(e.to_string())
    })?;
    let result = match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(crate::errors::AppError::Message(e.to_string())),
    };
    let mut guard = shadow_secrets()
        .lock()
        .map_err(|_| crate::errors::AppError::from("shadow keyring cache lock poisoned"))?;
    guard.remove(account);
    result
}
