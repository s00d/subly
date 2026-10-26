use chrono::Datelike;
use redb::WriteTransaction;
use tauri::Emitter;
use tauri::State;
use crate::AppState;
use crate::errors::{AppError, AppResult};
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

pub(crate) fn apply_expense_payment_record_kv_in_tx(
    tx: &WriteTransaction,
    old_row: Option<&ExpenseDoc>,
    new_row: Option<&ExpenseDoc>,
) -> Result<(), crate::errors::AppError> {
    use crate::state::{kv_delete_in_tx, kv_set_in_tx};
    if let Some(old) = old_row {
        if let Some(key) = expense_payment_record_index_key(&old.subscription_id, &old.payment_record_id) {
            kv_delete_in_tx(tx, &key)?;
        }
    }
    if let Some(new_row) = new_row {
        if let Some(key) = expense_payment_record_index_key(&new_row.subscription_id, &new_row.payment_record_id) {
            kv_set_in_tx(tx, &key, &new_row.id)?;
        }
    }
    Ok(())
}

pub(crate) fn update_expense_payment_record_index(
    guard: &mut crate::state::AppStateInner,
    old_row: Option<&ExpenseDoc>,
    new_row: Option<&ExpenseDoc>,
) -> Result<(), crate::errors::AppError> {
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

pub(crate) fn emit_expenses_changed(app: &tauri::AppHandle, action: &str) {
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({
            "entity": "expenses",
            "action": action
        }),
    );
}

fn expense_row_from_input(mut input: ExpenseInputDto) -> AppResult<ExpenseDoc> {
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
    /// Catalog id -> name (categories + payment methods) so search matches display names.
    catalog_names: std::collections::HashMap<String, String>,
}

impl ExpenseListFilter {
    fn from_request(req: &ExpensesPageRequestDto, data: &crate::AppDataDoc) -> Self {
        let catalog_names = data
            .categories
            .iter()
            .map(|c| (c.id.clone(), c.name.clone()))
            .chain(data.payment_methods.iter().map(|p| (p.id.clone(), p.name.clone())))
            .collect();
        Self {
            catalog_names,
            search: req.search.trim().to_lowercase(),
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
            let name_of = |id: &str| self.catalog_names.get(id).map(String::as_str).unwrap_or("");
            let haystack = format!(
                "{} {} {} {} {} {}",
                e.name,
                e.notes,
                e.category_id,
                e.payment_method_id,
                name_of(&e.category_id),
                name_of(&e.payment_method_id),
            )
            .to_lowercase();
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
) -> AppResult<Vec<ExpenseDoc>> {
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
        guard.table_list_typed(EntityTable::Expenses)}
}

#[tauri::command]
pub fn list_expenses_page(
    state: State<'_, AppState>,
    request: Option<ExpensesPageRequestDto>,
) -> Result<ExpensesPageResponseDto, crate::errors::AppError> {
    (|| -> AppResult<ExpensesPageResponseDto> {
        let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let req = request.unwrap_or_default();
        let filter = ExpenseListFilter::from_request(&req, &guard.app_data);
        let rates = crate::rate_map(&guard.app_data);
        let mut items: Vec<ExpenseDoc> =
            candidate_expenses_for_date_filter(&guard, filter.date_from_nd, filter.date_to_nd)?;
        drop(guard);

        items.retain(|e| filter.matches(e));

        let in_main = |e: &ExpenseDoc| crate::convert_to_main(e.amount, &e.currency_id, &rates);
        match req.sort_by.as_str() {
            "amount_desc" => items.sort_by_key(|e| std::cmp::Reverse(in_main(e))),
            "amount_asc" => items.sort_by_key(in_main),
            "date_asc" => items.sort_by_key(expense_sort_timestamp),
            _ => items.sort_by_key(|b| std::cmp::Reverse(expense_sort_timestamp(b))),
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
    })()
}

#[tauri::command]
pub fn expenses_total_filtered(
    state: State<'_, AppState>,
    filter: ExpensesPageRequestDto,
) -> Result<String, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let total = expenses_total_in_main(&guard, &filter)?;
    Ok(crate::money::to_ipc_string(&total))
}

/// Filtered expense total, every amount converted to the main currency.
fn expenses_total_in_main(
    guard: &crate::state::AppStateInner,
    filter: &ExpensesPageRequestDto,
) -> AppResult<rust_decimal::Decimal> {
    let list_filter = ExpenseListFilter::from_request(filter, &guard.app_data);
    let rates = crate::rate_map(&guard.app_data);
    let items: Vec<ExpenseDoc> =
        candidate_expenses_for_date_filter(guard, list_filter.date_from_nd, list_filter.date_to_nd)?;
    Ok(items
        .iter()
        .filter(|e| list_filter.matches(e))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
        .sum())
}

#[tauri::command]
pub fn expenses_for_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<ExpenseDoc>, crate::errors::AppError> {
    (|| -> AppResult<Vec<ExpenseDoc>> {
        if !(1..=12).contains(&month) {
            return Err(AppError::Message("month must be between 1 and 12".to_string()));
        }
        let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let mut items: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
        drop(guard);
        items.retain(|e| {
            e.naive_date()
                .map(|d| d.year() == year && d.month() == month)
                .unwrap_or(false)
        });
        items.sort_by_key(expense_sort_timestamp);
        Ok(items)
    })()
}

#[tauri::command]
pub fn expenses_get_by_id(state: State<'_, AppState>, id: String) -> Result<Option<ExpenseDoc>, crate::errors::AppError> {
    (|| -> AppResult<Option<ExpenseDoc>> {
        let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        guard.table_get_expense_by_id(&id)})()
}

#[tauri::command]
pub fn expenses_insert(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        expense.validate()?;
        let expense = expense_row_from_input(expense)?;
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let existing = guard.table_get_expense_by_id(&expense.id)?;
        guard
            .table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
        update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
        update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
        drop(guard);
        emit_expenses_changed(&app, "insert");
        Ok(())
    })()
}

#[tauri::command]
pub fn expenses_upsert(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        expense.validate()?;
        let expense = expense_row_from_input(expense)?;
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let existing = guard.table_get_expense_by_id(&expense.id)?;
        guard
            .table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
        update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
        update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
        drop(guard);
        emit_expenses_changed(&app, "upsert");
        Ok(())
    })()
}

#[tauri::command]
pub fn expenses_update(app: tauri::AppHandle, state: State<'_, AppState>, expense: ExpenseInputDto) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        expense.validate()?;
        let expense = expense_row_from_input(expense)?;
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let existing = guard.table_get_expense_by_id(&expense.id)?;
        guard
            .table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
        update_expense_payment_record_index(&mut guard, existing.as_ref(), Some(&expense))?;
        update_expense_day_index(guard.db.as_ref(), existing.as_ref(), Some(&expense))?;
        drop(guard);
        emit_expenses_changed(&app, "update");
        Ok(())
    })()
}

#[tauri::command]
pub fn expenses_delete(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
        let existing = guard.table_get_expense_by_id(&id)?;
        guard.table_delete_by_id(EntityTable::Expenses, &id)?;
        update_expense_payment_record_index(&mut guard, existing.as_ref(), None)?;
        update_expense_day_index(guard.db.as_ref(), existing.as_ref(), None)?;
        drop(guard);
        emit_expenses_changed(&app, "delete");
        Ok(())
    })()
}

#[tauri::command]
pub fn expenses_delete_batch(app: tauri::AppHandle, state: State<'_, AppState>, ids: Vec<String>) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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
    })()
}

#[tauri::command]
pub fn expenses_delete_by_payment_record(app: tauri::AppHandle, state: State<'_, AppState>, sub_id: String, pr_id: String) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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
    })()
}

#[tauri::command]
pub fn expenses_count(state: State<'_, AppState>) -> Result<usize, crate::errors::AppError> {
    (|| -> AppResult<usize> {
        let arr = {
            let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
            guard.table_list_typed::<ExpenseDoc>(EntityTable::Expenses)?
        };
        Ok(arr.len())
    })()
}

#[tauri::command]
pub fn expenses_update_tags_batch(app: tauri::AppHandle, state: State<'_, AppState>, old_name: String, new_name: String) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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
    })()
}

#[tauri::command]
pub fn expenses_remove_tag_batch(app: tauri::AppHandle, state: State<'_, AppState>, tag_name: String) -> Result<(), crate::errors::AppError> {
    (|| -> AppResult<()> {
        let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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
    })()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expenses_total_converts_to_main_and_search_matches_catalog_names() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let mut doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        doc.currencies[1].rate = rust_decimal::Decimal::from(2);
        let mut usd = doc.expenses[0].clone();
        usd.id = "exp-usd".into();
        usd.amount = rust_decimal::Decimal::from(10);
        usd.currency_id = doc.currencies[1].id.clone();
        usd.category_id = doc.categories[1].id.clone();
        doc.expenses.push(usd);
        let category_name = doc.categories[1].name.clone();
        let state = crate::state::AppStateInner { db: std::sync::Arc::new(db), app_data: doc };

        // 7.5 (main) + 10 / 2
        let total = expenses_total_in_main(&state, &ExpensesPageRequestDto::default()).expect("total");
        assert_eq!(total, "12.5".parse::<rust_decimal::Decimal>().unwrap());

        let by_name = ExpensesPageRequestDto { search: category_name.to_uppercase(), ..Default::default() };
        assert_eq!(expenses_total_in_main(&state, &by_name).expect("total"), rust_decimal::Decimal::from(5));
    }

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
            amount: crate::money::from_f64_legacy(1.0),
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
