use tauri::State;
use crate::AppState;

const CONFIG_PREFIX: &str = "config:";

#[tauri::command]
pub fn config_get(state: State<'_, AppState>, key: String) -> Result<Option<serde_json::Value>, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let full_key = format!("{CONFIG_PREFIX}{key}");
    let raw = guard.redb_get(&full_key)?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(serde_json::Value::String(inner)) => {
            match serde_json::from_str::<serde_json::Value>(&inner) {
                Ok(unwrapped) => Ok(Some(unwrapped)),
                Err(_) => Ok(Some(serde_json::Value::String(inner))),
            }
        }
        Ok(v) => Ok(Some(v)),
        Err(_) => Ok(Some(serde_json::Value::String(raw))),
    }
}

#[tauri::command]
pub fn config_set(state: State<'_, AppState>, key: String, value: serde_json::Value) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let full_key = format!("{CONFIG_PREFIX}{key}");
    let raw = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    guard.redb_set(&full_key, &raw)
}

#[tauri::command]
pub fn config_delete(state: State<'_, AppState>, key: String) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let full_key = format!("{CONFIG_PREFIX}{key}");
    guard.redb_delete(&full_key)
}
