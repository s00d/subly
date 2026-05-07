use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn redb_get(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.redb_get(&key)
}

#[tauri::command]
pub fn redb_set(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.redb_set(&key, &value)
}

#[tauri::command]
pub fn redb_delete(state: State<'_, AppState>, key: String) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.redb_delete(&key)
}
