use tauri::State;
use crate::{AppState, RatePoint};

fn read_history(state: &State<'_, AppState>) -> Result<std::collections::HashMap<String, Vec<RatePoint>>, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let raw = crate::state::load_rate_history(guard.db.as_ref())?;
    serde_json::from_str(&raw).map_err(crate::errors::AppError::from)
}

fn save_history(state: &State<'_, AppState>, history: &std::collections::HashMap<String, Vec<RatePoint>>) -> Result<(), crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    crate::state::save_rate_history(
        guard.db.as_ref(),
        &serde_json::to_string(history)?,
    )
}

#[tauri::command]
pub fn get_rate_history_widget(
    state: State<'_, AppState>,
    target_ids: Vec<String>,
    days: u32,
) -> Result<std::collections::HashMap<String, Vec<RatePoint>>, crate::errors::AppError> {
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

/// Upserts today's point per currency (same-day reruns replace it) and drops points older than `keep_days`.
pub(crate) fn record_snapshots(
    history: &mut std::collections::HashMap<String, Vec<RatePoint>>,
    points: &[(String, rust_decimal::Decimal)],
    today: chrono::NaiveDate,
    keep_days: i64,
) {
    let today_s = today.to_string();
    let oldest = (today - chrono::Duration::days(keep_days)).to_string();
    for (id, rate) in points {
        let series = history.entry(id.clone()).or_default();
        series.retain(|p| p.recorded_at != today_s);
        series.push(RatePoint { rate: *rate, recorded_at: today_s.clone() });
    }
    for series in history.values_mut() {
        series.retain(|p| p.recorded_at.as_str() >= oldest.as_str());
    }
    history.retain(|_, v| !v.is_empty());
}

pub(crate) fn record_snapshots_in_db(
    db: &redb::Database,
    points: &[(String, rust_decimal::Decimal)],
    today: chrono::NaiveDate,
    keep_days: i64,
) -> Result<(), crate::errors::AppError> {
    let raw = crate::state::load_rate_history(db)?;
    let mut history: std::collections::HashMap<String, Vec<RatePoint>> = serde_json::from_str(&raw).unwrap_or_default();
    record_snapshots(&mut history, points, today, keep_days);
    crate::state::save_rate_history(db, &serde_json::to_string(&history)?)
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn record_snapshots_dedupes_day_and_prunes() {
        let day = |s: &str| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        let mut h = std::collections::HashMap::new();
        h.insert("cur-x".to_string(), vec![RatePoint { rate: 1.into(), recorded_at: "2026-01-01".into() }]);
        record_snapshots(&mut h, &[("cur-a".into(), 2.into())], day("2026-05-01"), 30);
        record_snapshots(&mut h, &[("cur-a".into(), 3.into())], day("2026-05-01"), 30);
        assert_eq!(h["cur-a"].len(), 1, "same day replaced");
        assert_eq!(h["cur-a"][0].rate, rust_decimal::Decimal::from(3));
        assert!(!h.contains_key("cur-x"), "older than keep_days pruned");
    }
}

#[tauri::command]
pub fn rate_history_save_snapshot(
    state: State<'_, AppState>,
    currency_id: String,
    rate: String,
) -> Result<(), crate::errors::AppError> {
    let rate = crate::money::parse_money_str(&rate).map_err(crate::errors::AppError::from)?;
    let mut history = read_history(&state)?;
    let today = chrono::Local::now().date_naive().to_string();
    history
        .entry(currency_id)
        .or_default()
        .push(RatePoint {
            rate,
            recorded_at: today,
        });
    save_history(&state, &history)
}

#[tauri::command]
pub fn rate_history_get(state: State<'_, AppState>, currency_id: String, days: u32) -> Result<Vec<RatePoint>, crate::errors::AppError> {
    let v = get_rate_history_widget(state, vec![currency_id.clone()], days)?;
    Ok(v.get(currency_id.as_str()).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn rate_history_prune(state: State<'_, AppState>, keep_days: i64) -> Result<usize, crate::errors::AppError> {
    let mut h = read_history(&state)?;
    let since = chrono::Local::now().date_naive() - chrono::Duration::days(keep_days);
    let mut removed = 0usize;
    for v in h.values_mut() {
        let before = v.len();
        v.retain(|p| chrono::NaiveDate::parse_from_str(&p.recorded_at, "%Y-%m-%d").map(|d| d >= since).unwrap_or(false));
        removed += before - v.len();
    }
    save_history(&state, &h)?;
    Ok(removed)
}

#[tauri::command]
pub fn rate_history_clear(state: State<'_, AppState>) -> Result<(), crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    crate::state::save_rate_history(guard.db.as_ref(), "{}")
}

#[tauri::command]
pub fn rate_history_count(state: State<'_, AppState>) -> Result<usize, crate::errors::AppError> {
    let h = read_history(&state)?;
    Ok(h.values().map(|v| v.len()).sum())
}
