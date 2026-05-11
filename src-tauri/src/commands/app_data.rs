use tauri::{Emitter, State};
use redb::ReadableTable;
use crate::AppState;
use crate::models::{AppConfigDoc, AppDataDoc};
use crate::state::{read_singleton_bin_typed, write_singleton_bin_typed, T2_CONFIG};
use crate::commands::seed::seed_get_default_data;

#[tauri::command]
pub fn load_all_data(state: State<'_, AppState>) -> Result<AppDataDoc, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    guard.doc()
}

#[tauri::command]
pub fn load_app_data(state: State<'_, AppState>) -> Result<AppDataDoc, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    guard.doc()
}

#[tauri::command]
pub fn reset_app_data(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppDataDoc, crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let next = seed_get_default_data()?;
    guard.apply_snapshot_typed(&next)?;
    clear_kv_by_prefix(guard.db.as_ref(), "config:")?;
    let settings_blob = serde_json::to_string(&crate::build_default_ui_settings(&next))
        .map_err(|e| e.to_string())?;
    guard.redb_set("config:settings", &settings_blob)?;

    let mut cfg: AppConfigDoc = read_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, AppConfigDoc::default())?;
    cfg.initialized = true;
    write_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, &cfg)?;
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "appData", "action": "reset" }),
    );
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "settings", "action": "reset" }),
    );
    Ok(next)
}

fn clear_kv_by_prefix(db: &redb::Database, prefix: &str) -> Result<(), crate::errors::AppError> {
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(crate::KV_TABLE)?;
        let mut keys: Vec<String> = Vec::new();
        let iter = table.iter()?;
        for row in iter {
            let (key, _) = row?;
            let key_str = key.value();
            if key_str.starts_with(prefix) {
                keys.push(key_str.to_string());
            }
        }
        for key in keys {
            let _ = table.remove(key.as_str())?;
        }
    }
    tx.commit()?;
    Ok(())
}
