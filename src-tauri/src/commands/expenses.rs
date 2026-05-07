use chrono::Datelike;
use tauri::Emitter;
use tauri::State;
use crate::AppState;
use crate::models::{
    ExpenseDoc, ExpenseInputDto, ExpensesPageRequestDto, ExpensesPageResponseDto, normalize_expense_timestamp,
};
use crate::state::{expense_ids_in_day_range, update_expense_day_index, EntityTable};

fn expense_payment_record_index_key(subscription_id: &str, payment_record_id: &str) -> Option<String> {
    if subscription_id.trim().is_empty() || payment_record_id.trim().is_empty() {
        return None;
    }
    Some(format!(
        "idx:expense_by_payment_record:{}:{}",
        subscription_id, payment_record_id
    ))
}

fn update_expense_payment_record_index(
    guard: &mut crate::state::AppStateInner,
    old_row: Option<&ExpenseDoc>,
    new_row: Option<&ExpenseDoc>,
) -> Result<(), String> {
    if let Some(old) = old_row {
        if let Some(key) = expense_payment_record_index_key(&old.subscription_id, &old.payment_record_id) {
            guard.redb_delete(&key)?;
        }
    }
    if let Some(new_row) = new_row {
        if let Some(key) = expense_payment_record_index_key(&new_row.subscription_id, &new_row.payment_record_id) {
            guard.redb_set(&key, &new_row.id)?;
        }
    }
    Ok(())
}

fn emit_expenses_changed(app: &tauri::AppHandle, action: &str) {
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({
            "entity": "expenses",
            "action": action
        }),
    );
}

fn expense_row_from_input(mut input: ExpenseInputDto) -> Result<ExpenseDoc, String> {
    if input.id.trim().is_empty() {
        input.id = format!("exp-{}", chrono::Utc::now().timestamp_millis());
    }
    let created_at = if input.created_at.trim().is_empty() {
        chrono::Utc::now().to_rfc3339()
    } else {
        normalize_expense_timestamp(&input.created_at)?
    };
    Ok(ExpenseDoc {
        id: input.id,
        updated_at: chrono::Utc::now().timestamp_millis(),
        name: input.name.trim().to_string(),
        amount: input.amount,
        currency_id: input.currency_id.trim().to_string(),
        created_at,
        category_id: input.category_id,
        tags: input.tags,
        payment_method_id: input.payment_method_id,
        payer_user_id: input.payer_user_id,
        notes: input.notes,
        url: input.url,
        subscription_id: input.subscription_id,
        payment_record_id: input.payment_record_id,
    })
}

fn expense_sort_timestamp(e: &ExpenseDoc) -> i64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(e.created_at.trim()) {
        return dt.timestamp();
    }
    if let Ok((y, m, d)) = crate::models::parse_loose_date_to_ymd(&e.created_at) {
        if let Some(nd) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
            if let Some(ndt) = nd.and_hms_opt(12, 0, 0) {
                return chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc).timestamp();
            }
        }
    }
    0
}

fn filter_bound_nd(raw: &str) -> Option<chrono::NaiveDate> {
    if raw.trim().is_empty() {
        return None;
    }
    let (y, m, d) = crate::models::parse_loose_date_to_ymd(raw).ok()?;
    chrono::NaiveDate::from_ymd_opt(y, m, d)
}

/// Shared search/category/tag/date filtering for list + totals (see [`list_expenses_page`], [`expenses_total_filtered`]).
struct ExpenseListFilter {
    search: String,
    category: String,
    payment_method: String,
    tag: String,
    date_from_nd: Option<chrono::NaiveDate>,
    date_to_nd: Option<chrono::NaiveDate>,
}

impl ExpenseListFilter {
    fn from_request(req: &ExpensesPageRequestDto) -> Self {
        Self {
            search: req.search.trim().to_ascii_lowercase(),
            category: req.category_id.trim().to_string(),
            payment_method: req.payment_method_id.trim().to_string(),
            tag: req.tag.trim().to_ascii_lowercase(),
            date_from_nd: filter_bound_nd(&req.date_from),
            date_to_nd: filter_bound_nd(&req.date_to),
        }
    }

    fn matches(&self, e: &ExpenseDoc) -> bool {
        let Some(expense_nd) = e.naive_date() else {
            return false;
        };
        if !self.category.is_empty() && e.category_id != self.category {
            return false;
        }
        if !self.payment_method.is_empty() && e.payment_method_id != self.payment_method {
            return false;
        }
        if !self.tag.is_empty() && !e.tags.iter().any(|t| t.to_ascii_lowercase() == self.tag) {
            return false;
        }
        if self.date_from_nd.map(|d| expense_nd < d).unwrap_or(false) {
            return false;
        }
        if self.date_to_nd.map(|d| expense_nd > d).unwrap_or(false) {
            return false;
        }
        if !self.search.is_empty() {
            let haystack =
                format!("{} {} {} {}", e.name, e.notes, e.category_id, e.payment_method_id).to_ascii_lowercase();
            if !haystack.contains(&self.search) {
                return false;
            }
        }
        true
    }
}

fn candidate_expenses_for_date_filter(
    guard: &crate::state::AppStateInner,
    date_from_nd: Option<chrono::NaiveDate>,
    date_to_nd: Option<chrono::NaiveDate>,
) -> Result<Vec<ExpenseDoc>, String> {
    if let (Some(df), Some(dt)) = (date_from_nd, date_to_nd) {
        let ids = expense_ids_in_day_range(guard.db.as_ref(), df, dt)?;
        Ok(guard
            .app_data
            .expenses
            .iter()
            .filter(|e| ids.contains(&e.id))
            .cloned()
            .collect())
    } else {
        guard.table_list_typed(EntityTable::Expenses)
    }
}

#[tauri::command]
pub fn list_expenses_page(
    state: State<'_, AppState>,
    request: Option<ExpensesPageRequestDto>,
) -> Result<ExpensesPageResponseDto, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let req = request.unwrap_or_default();
    let filter = ExpenseListFilter::from_request(&req);
    let mut items: Vec<ExpenseDoc> =
        candidate_expenses_for_date_filter(&guard, filter.date_from_nd, filter.date_to_nd)?;
    drop(guard);

    items.retain(|e| filter.matches(e));

    match req.sort_by.as_str() {
        "amount_desc" => {
            items.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));
        }
        "amount_asc" => {
            items.sort_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal));
        }
        "date_asc" => items.sort_by(|a, b| expense_sort_timestamp(a).cmp(&expense_sort_timestamp(b))),
        _ => items.sort_by(|a, b| expense_sort_timestamp(b).cmp(&expense_sort_timestamp(a))),
    }

    let total = items.len();
    let limit = req.limit;
    let offset = req.offset.min(total);
    let paged = if limit == 0 {
        items.into_iter().skip(offset).collect::<Vec<_>>()
    } else {
        items.into_iter().skip(offset).take(limit).collect::<Vec<_>>()
    };

    Ok(ExpensesPageResponseDto { items: paged, total })
}

#[tauri::command]
pub fn expenses_total_filtered(
    state: State<'_, AppState>,
    filter: ExpensesPageRequestDto,
) -> Result<f64, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let list_filter = ExpenseListFilter::from_request(&filter);
    let items: Vec<ExpenseDoc> =
        candidate_expenses_for_date_filter(&guard, list_filter.date_from_nd, list_filter.date_to_nd)?;
    drop(guard);

    let total: f64 = items.iter().filter(|e| list_filter.matches(e)).map(|e| e.amount).sum();
    Ok(total)
}

#[tauri::command]
pub fn expenses_for_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<ExpenseDoc>, String> {
    if !(1..=12).contains(&month) {
        return Err("month must be between 1 and 12".to_string());
    }
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let mut items: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    drop(guard);
    items.retain(|e| {
        e.naive_date()
            .map(|d| d.year() == year && d.month() == month)
            .unwrap_or(false)
    });
    items.sort_by(|a, b| expense_sort_timestamp(a).cmp(&expense_sort_timestamp(b)));
    Ok(items)
}

#[tauri::command]
pub fn expenses_get_by_id(state: State<'_, AppState>, id: String) -> Result<Option<ExpenseDoc>, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_get_expense_by_id(&id)
}

#[tauri::command]
pub fn expenses_insert(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), String> {
    expense.validate()?;
    let expense = expense_row_from_input(expense)?;
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let existing = guard.table_get_expense_by_id(&expense.id)?;
    guard.table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
    update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
    update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
    drop(guard);
    emit_expenses_changed(&app, "insert");
    Ok(())
}

#[tauri::command]
pub fn expenses_upsert(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), String> {
    expense.validate()?;
    let expense = expense_row_from_input(expense)?;
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let existing = guard.table_get_expense_by_id(&expense.id)?;
    guard.table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
    update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
    update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
    drop(guard);
    emit_expenses_changed(&app, "upsert");
    Ok(())
}

#[tauri::command]
pub fn expenses_update(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), String> {
    expense.validate()?;
    let expense = expense_row_from_input(expense)?;
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let existing = guard.table_get_expense_by_id(&expense.id)?;
    guard.table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
    update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
    update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
    drop(guard);
    emit_expenses_changed(&app, "update");
    Ok(())
}

#[tauri::command]
pub fn expenses_delete(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let existing = guard.table_get_expense_by_id(&id)?;
    guard.table_delete_by_id(EntityTable::Expenses, &id)?;
    update_expense_payment_record_index(&mut guard, existing.as_ref(), None)?;
    update_expense_day_index(guard.db.as_ref(), existing.as_ref(), None)?;
    drop(guard);
    emit_expenses_changed(&app, "delete");
    Ok(())
}

#[tauri::command]
pub fn expenses_delete_batch(app: tauri::AppHandle, state: State<'_, AppState>, ids: Vec<String>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    for id in ids {
        if !id.trim().is_empty() {
            let existing = guard.table_get_expense_by_id(&id)?;
            guard.table_delete_by_id(EntityTable::Expenses, &id)?;
            update_expense_payment_record_index(&mut guard, existing.as_ref(), None)?;
            update_expense_day_index(guard.db.as_ref(), existing.as_ref(), None)?;
        }
    }
    drop(guard);
    emit_expenses_changed(&app, "delete_batch");
    Ok(())
}

#[tauri::command]
pub fn expenses_delete_by_payment_record(app: tauri::AppHandle, state: State<'_, AppState>, sub_id: String, pr_id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    if let Some(idx_key) = expense_payment_record_index_key(&sub_id, &pr_id) {
        if let Some(expense_id) = guard.redb_get(&idx_key)? {
            if let Some(expense) = guard.table_get_expense_by_id(&expense_id)? {
                guard.table_delete_by_id(EntityTable::Expenses, &expense.id)?;
                update_expense_payment_record_index(&mut guard, Some(&expense), None)?;
                update_expense_day_index(guard.db.as_ref(), Some(&expense), None)?;
                drop(guard);
                emit_expenses_changed(&app, "delete_by_payment_record");
                return Ok(());
            }
        }
    }

    // Fallback for legacy/stale index state.
    let arr: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    for expense in arr {
        if expense.subscription_id == sub_id && expense.payment_record_id == pr_id {
            guard.table_delete_by_id(EntityTable::Expenses, &expense.id)?;
            update_expense_payment_record_index(&mut guard, Some(&expense), None)?;
            update_expense_day_index(guard.db.as_ref(), Some(&expense), None)?;
        }
    }
    drop(guard);
    emit_expenses_changed(&app, "delete_by_payment_record");
    Ok(())
}

#[tauri::command]
pub fn expenses_count(state: State<'_, AppState>) -> Result<usize, String> {
    let arr = {
        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
        guard.table_list_typed::<ExpenseDoc>(EntityTable::Expenses)?
    };
    Ok(arr.len())
}

#[tauri::command]
pub fn expenses_update_tags_batch(app: tauri::AppHandle, state: State<'_, AppState>, old_name: String, new_name: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let mut arr: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    for e in &mut arr {
        let mut changed = false;
        for tag in &mut e.tags {
            if tag == &old_name {
                *tag = new_name.clone();
                changed = true;
            }
        }
        if changed {
            let before = guard.table_get_expense_by_id(&e.id)?;
            guard.table_upsert_typed(EntityTable::Expenses, e, &e.id)?;
            update_expense_day_index(guard.db.as_ref(), before.as_ref(), Some(e))?;
        }
    }
    drop(guard);
    emit_expenses_changed(&app, "update_tags_batch");
    Ok(())
}

#[tauri::command]
pub fn expenses_remove_tag_batch(app: tauri::AppHandle, state: State<'_, AppState>, tag_name: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let mut arr: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    for e in &mut arr {
        let before = e.tags.len();
        e.tags.retain(|t| t != &tag_name);
        if e.tags.len() != before {
            let old_row = guard.table_get_expense_by_id(&e.id)?;
            guard.table_upsert_typed(EntityTable::Expenses, e, &e.id)?;
            update_expense_day_index(guard.db.as_ref(), old_row.as_ref(), Some(e))?;
        }
    }
    drop(guard);
    emit_expenses_changed(&app, "remove_tag_batch");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expense_payment_record_index_tracks_row_lifecycle() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let mut state = crate::state::AppStateInner {
            db: std::sync::Arc::new(db),
            app_data: doc,
        };

        let row = ExpenseDoc {
            id: "exp-index-test".to_string(),
            updated_at: 1,
            name: "idx".to_string(),
            amount: 1.0,
            currency_id: "cur-2".to_string(),
            created_at: "2026-01-01T12:00:00.000Z".to_string(),
            category_id: "cat-1".to_string(),
            tags: vec![],
            payment_method_id: String::new(),
            payer_user_id: String::new(),
            notes: String::new(),
            url: String::new(),
            subscription_id: "sub-1".to_string(),
            payment_record_id: "pr-1".to_string(),
        };

        update_expense_payment_record_index(&mut state, None, Some(&row)).expect("set");
        let idx_key = expense_payment_record_index_key("sub-1", "pr-1").expect("idx key");
        assert_eq!(
            state.redb_get(&idx_key).expect("read"),
            Some("exp-index-test".to_string())
        );

        update_expense_payment_record_index(&mut state, Some(&row), None).expect("clear");
        assert!(state.redb_get(&idx_key).expect("read cleared").is_none());
    }
}
