const SECURE_PREFIX: &str = "secure_storage.";

fn storage_account(raw_key: &str) -> String {
    format!("{SECURE_PREFIX}{}", raw_key.trim())
}

#[tauri::command]
pub fn secure_storage_set(key: String, value: String) -> Result<(), String> {
    crate::keyring_store::set(&storage_account(&key), &value)
}

#[tauri::command]
pub fn secure_storage_get(key: String) -> Result<Option<String>, String> {
    crate::keyring_store::get(&storage_account(&key))
}

#[tauri::command]
pub fn secure_storage_delete(key: String) -> Result<(), String> {
    crate::keyring_store::delete(&storage_account(&key))
}
