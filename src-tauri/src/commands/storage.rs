use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn redb_get(state: State<'_, AppState>, key: String) -> Result<Option<String>, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    guard.redb_get(&key)
}

#[tauri::command]
pub fn redb_set(state: State<'_, AppState>, key: String, value: String) -> Result<(), crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    guard.redb_set(&key, &value)
}

#[tauri::command]
pub fn redb_delete(state: State<'_, AppState>, key: String) -> Result<(), crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    guard.redb_delete(&key)
}
