use tauri::{Emitter, State};
use crate::AppState;
use crate::models::{AppConfigDoc, AppDataDoc};
use crate::state::{read_singleton_bin_typed, write_singleton_bin_typed, T2_CONFIG};
use crate::commands::seed::seed_get_default_data;

#[tauri::command]
pub fn load_all_data(state: State<'_, AppState>) -> Result<AppDataDoc, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.doc()
}

#[tauri::command]
pub fn load_app_data(state: State<'_, AppState>) -> Result<AppDataDoc, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.doc()
}

#[tauri::command]
pub fn reset_app_data(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppDataDoc, String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let next = seed_get_default_data()?;
    guard.apply_snapshot_typed(&next)?;

    let mut cfg: AppConfigDoc = read_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, AppConfigDoc::default())?;
    cfg.initialized = true;
    write_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, &cfg)?;
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "appData", "action": "reset" }),
    );
    Ok(next)
}
