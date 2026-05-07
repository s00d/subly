use tauri::State;
use crate::{AppState, RatePoint};

fn read_history(state: &State<'_, AppState>) -> Result<std::collections::HashMap<String, Vec<RatePoint>>, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let raw = crate::state::load_rate_history(guard.db.as_ref())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_history(state: &State<'_, AppState>, history: &std::collections::HashMap<String, Vec<RatePoint>>) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    crate::state::save_rate_history(
        guard.db.as_ref(),
        &serde_json::to_string(history).map_err(|e| e.to_string())?,
    )
}

#[tauri::command]
pub fn get_rate_history_widget(
    state: State<'_, AppState>,
    target_ids: Vec<String>,
    days: u32,
) -> Result<std::collections::HashMap<String, Vec<RatePoint>>, String> {
    let history = read_history(&state)?;
    if target_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let since = chrono::Local::now().date_naive() - chrono::Duration::days(days as i64);
    let mut result = std::collections::HashMap::new();
    for id in target_ids {
        let points = history.get(&id).cloned().unwrap_or_default();
        let filtered = points.into_iter().filter(|p| {
            chrono::NaiveDate::parse_from_str(&p.recorded_at, "%Y-%m-%d")
                .map(|d| d >= since)
                .unwrap_or(false)
        }).collect::<Vec<_>>();
        result.insert(id, filtered);
    }
    Ok(result)
}

#[tauri::command]
pub fn rate_history_save_snapshot(state: State<'_, AppState>, currency_id: String, rate: f64) -> Result<(), String> {
    let mut history = read_history(&state)?;
    let today = chrono::Local::now().date_naive().to_string();
    history.entry(currency_id).or_default().push(RatePoint { rate, recorded_at: today });
    save_history(&state, &history)
}

#[tauri::command]
pub fn rate_history_get(state: State<'_, AppState>, currency_id: String, days: u32) -> Result<Vec<RatePoint>, String> {
    let v = get_rate_history_widget(state, vec![currency_id.clone()], days)?;
    Ok(v.get(currency_id.as_str()).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn rate_history_prune(state: State<'_, AppState>, keep_days: i64) -> Result<usize, String> {
    let mut h = read_history(&state)?;
    let since = chrono::Local::now().date_naive() - chrono::Duration::days(keep_days);
    let mut removed = 0usize;
    for (_k, v) in h.iter_mut() {
        let before = v.len();
        v.retain(|p| chrono::NaiveDate::parse_from_str(&p.recorded_at, "%Y-%m-%d").map(|d| d >= since).unwrap_or(false));
        removed += before - v.len();
    }
    save_history(&state, &h)?;
    Ok(removed)
}

#[tauri::command]
pub fn rate_history_clear(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    crate::state::save_rate_history(guard.db.as_ref(), "{}")
}

#[tauri::command]
pub fn rate_history_count(state: State<'_, AppState>) -> Result<usize, String> {
    let h = read_history(&state)?;
    Ok(h.values().map(|v| v.len()).sum())
}
