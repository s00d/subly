use chrono::{Datelike, Duration, Local, NaiveDate};
use std::collections::HashMap;
use tauri::Emitter;
use tauri::State;
use crate::AppState;
use crate::models::{
    ExpenseDoc, PaymentRecordDto, SubscriptionCredentialsMetaDto, SubscriptionDoc,
    SubscriptionInputDto, SubscriptionListItemDto, SubscriptionsPageRequestDto,
};
use crate::commands::expenses::{apply_expense_payment_record_kv_in_tx, emit_expenses_changed};
use crate::commands::subscription_credentials::{
    credentials_apply_optional, credentials_delete, read_meta_index,
};
use crate::state::{
    encode_bin, entity_table_upsert_bin_in_tx, rewrite_expense_day_index_in_tx, run_write_transaction,
    EntityTable,
};
use crate::state_tables::touch_expense_doc;

fn payment_record_index_key(payment_record_id: &str) -> String {
    format!("idx:payment_record:{}", payment_record_id)
}

fn sync_subscription_payment_record_index(
    guard: &mut crate::state::AppStateInner,
    old_row: Option<&SubscriptionDoc>,
    new_row: Option<&SubscriptionDoc>,
) -> Result<(), crate::errors::AppError> {
    if let Some(old) = old_row {
        for record in &old.payment_history {
            guard.redb_delete(&payment_record_index_key(&record.id))?;
        }
    }
    if let Some(new_row) = new_row {
        for record in &new_row.payment_history {
            guard.redb_set(&payment_record_index_key(&record.id), &new_row.id)?;
        }
    }
    Ok(())
}

/// Clear subscription/payment links on expenses tied to this subscription (keeps expense rows).
fn unlink_expenses_for_subscription(
    guard: &mut crate::state::AppStateInner,
    subscription_id: &str,
) -> Result<bool, crate::errors::AppError> {
    let ids: Vec<String> = guard
        .app_data
        .expenses
        .iter()
        .filter(|e| e.subscription_id == subscription_id)
        .map(|e| e.id.clone())
        .collect();
    let mut updates: Vec<(ExpenseDoc, ExpenseDoc)> = Vec::new();
    for exp_id in ids {
        let Some(existing) = guard.table_get_by_id_typed::<ExpenseDoc>(EntityTable::Expenses, &exp_id)? else {
            continue;
        };
        let mut exp = existing.clone();
        exp.subscription_id.clear();
        exp.payment_record_id.clear();
        let exp = touch_expense_doc(exp);
        updates.push((existing, exp));
    }
    if updates.is_empty() {
        return Ok(false);
    }

    let mut full_expenses = guard.app_data.expenses.clone();
    for (_, new_row) in &updates {
        if let Some(i) = full_expenses.iter().position(|e| e.id == new_row.id) {
            full_expenses[i] = new_row.clone();
        }
    }

    let db = guard.db.clone();
    run_write_transaction(db.as_ref(), |tx| {
        for (_, new_row) in &updates {
            let payload = encode_bin(new_row)?;
            entity_table_upsert_bin_in_tx(tx, EntityTable::Expenses, &new_row.id, &payload)?;
        }
        for (old, new) in &updates {
            apply_expense_payment_record_kv_in_tx(tx, Some(old), Some(new))?;
        }
        rewrite_expense_day_index_in_tx(tx, &full_expenses)?;
        Ok(())
    })?;

    guard.app_data.expenses = full_expenses;
    Ok(true)
}

fn emit_subscriptions_changed(app: &tauri::AppHandle, action: &str) {
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({
            "entity": "subscriptions",
            "action": action
        }),
    );
    #[cfg(target_os = "ios")]
    crate::widget_snapshot::export_ios_widget_snapshot_from_app(app);
}

fn subscription_row_from_input(mut input: SubscriptionInputDto) -> Result<SubscriptionDoc, crate::errors::AppError> {
    if input.id.trim().is_empty() {
        input.id = format!("sub-{}", chrono::Utc::now().timestamp_millis());
    }
    let created_at = if input.created_at.trim().is_empty() {
        chrono::Utc::now().to_rfc3339()
    } else {
        input.created_at.clone()
    };
    Ok(SubscriptionDoc {
        id: input.id,
        updated_at: chrono::Utc::now().timestamp_millis(),
        name: input.name.trim().to_string(),
        logo: input.logo,
        price: input.price,
        currency_id: input.currency_id.trim().to_string(),
        next_payment: normalize_date_ymd(&input.next_payment)?,
        start_date: normalize_date_ymd(&input.start_date)?,
        cycle: input.cycle,
        frequency: input.frequency.max(1),
        inactive: input.inactive,
        category_id: input.category_id,
        payment_method_id: input.payment_method_id,
        payer_user_id: input.payer_user_id,
        cancellation_date: input.cancellation_date,
        notes: input.notes,
        notify: input.notify,
        notify_days_before: input.notify_days_before,
        last_notified_date: input.last_notified_date,
        auto_renew: input.auto_renew,
        url: input.url,
        replacement_subscription_id: input.replacement_subscription_id,
        created_at,
        tags: input.tags,
        favorite: input.favorite,
        payment_history: input
            .payment_history
            .into_iter()
            .map(|mut r| {
                let trimmed = r.date.trim();
                if !trimmed.is_empty() {
                    r.date = normalize_date_ymd(trimmed)?;
                }
                Ok(r)
            })
            .collect::<Result<Vec<PaymentRecordDto>, crate::errors::AppError>>()?,
    })
}

fn normalize_date_ymd(raw: &str) -> Result<String, crate::errors::AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(crate::errors::AppError::from("field_invalid_date:nextPayment"));
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(date.format("%Y-%m-%d").to_string());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.date_naive().format("%Y-%m-%d").to_string());
    }
    if trimmed.len() >= 10 {
        if let Ok(date) = NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d") {
            return Ok(date.format("%Y-%m-%d").to_string());
        }
    }
    Err(crate::errors::AppError::from("field_invalid_date:nextPayment"))
}

fn monthly_price(cycle: u64, frequency: u64, price: f64) -> f64 {
    let freq = frequency.max(1) as f64;
    match cycle {
        1 => price * (30.0 / freq),
        2 => price * ((52.0 / 12.0) / freq),
        3 => price * (1.0 / freq),
        4 => price / (12.0 * freq),
        _ => price,
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    for d in (28..=31).rev() {
        if NaiveDate::from_ymd_opt(year, month, d).is_some() {
            return d;
        }
    }
    28
}

fn add_months_preserving_end(date: NaiveDate, months_delta: i32) -> Option<NaiveDate> {
    let source_day = date.day();
    let source_last = last_day_of_month(date.year(), date.month());
    let is_month_end = source_day == source_last;

    let total_months = date.year() * 12 + (date.month() as i32 - 1) + months_delta;
    let target_year = total_months.div_euclid(12);
    let target_month = (total_months.rem_euclid(12) + 1) as u32;
    let target_last = last_day_of_month(target_year, target_month);
    let target_day = if is_month_end {
        target_last
    } else {
        source_day.min(target_last)
    };
    NaiveDate::from_ymd_opt(target_year, target_month, target_day)
}

fn add_cycle_increment(date: NaiveDate, cycle: u64, frequency: u64, direction: i64) -> Option<NaiveDate> {
    let freq = frequency.max(1) as i64;
    match cycle {
        1 => Some(date + Duration::days(freq * direction)),
        2 => Some(date + Duration::days(7 * freq * direction)),
        3 => add_months_preserving_end(date, (freq as i32) * direction as i32),
        4 => add_months_preserving_end(date, (12 * freq as i32) * direction as i32),
        _ => Some(date),
    }
}

fn parse_subscription_date(raw: &str) -> Option<NaiveDate> {
    normalize_date_ymd(raw)
        .ok()
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
}

fn jump_to_window_start(
    anchor: NaiveDate,
    cycle: u64,
    frequency: u64,
    start: NaiveDate,
    end: NaiveDate,
) -> Option<NaiveDate> {
    let mut candidate = anchor;

    // Move backward/forward from anchor so historical months are supported too.
    if candidate > end {
        for _ in 0..400 {
            if candidate <= end {
                break;
            }
            candidate = add_cycle_increment(candidate, cycle, frequency, -1)?;
        }
    } else if candidate < start {
        for _ in 0..400 {
            if candidate >= start {
                break;
            }
            candidate = add_cycle_increment(candidate, cycle, frequency, 1)?;
        }
    }

    while candidate < start {
        candidate = add_cycle_increment(candidate, cycle, frequency, 1)?;
    }

    if candidate >= start && candidate <= end {
        Some(candidate)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_subscriptions_page(
    state: State<'_, AppState>,
    request: Option<SubscriptionsPageRequestDto>,
) -> Result<Vec<SubscriptionListItemDto>, crate::errors::AppError> {
    let guard = state
        .lock()
        .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let subscriptions = guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?;
    let today = Local::now().date_naive();
    let req = request.unwrap_or_default();
    let search = req.search.trim().to_ascii_lowercase();
    let state_filter = req.state.trim().to_ascii_lowercase();
    let category_filter = req.category_id.trim().to_string();
    let payment_filter = req.payment_method_id.trim().to_string();
    let tag_filter = req.tag.trim().to_ascii_lowercase();
    let sort_by = if req.sort_by.trim().is_empty() {
        "next_payment".to_string()
    } else {
        req.sort_by.trim().to_ascii_lowercase()
    };
    let mut rows = subscriptions
        .into_iter()
        .filter(|item| {
            if req.hide_disabled && item.inactive {
                return false;
            }
            if state_filter == "active" && item.inactive {
                return false;
            }
            if state_filter == "inactive" && !item.inactive {
                return false;
            }
            if !category_filter.is_empty() && item.category_id != category_filter {
                return false;
            }
            if !payment_filter.is_empty() && item.payment_method_id != payment_filter {
                return false;
            }
            if !tag_filter.is_empty() && !item.tags.iter().any(|t| t.to_ascii_lowercase() == tag_filter) {
                return false;
            }
            if !search.is_empty() {
                let haystack = format!("{} {} {} {}", item.name, item.notes, item.category_id, item.payment_method_id).to_ascii_lowercase();
                if !haystack.contains(&search) {
                    return false;
                }
            }
            true
        })
        .map(|item| {
            let monthly = monthly_price(item.cycle as u64, item.frequency as u64, item.price);
            let (days_left, overdue) = NaiveDate::parse_from_str(&item.next_payment, "%Y-%m-%d")
            .map(|next| {
                let diff = (next - today).num_days();
                let clamped = diff.clamp(0, 30) as u64;
                let is_overdue = !item.inactive && !item.auto_renew && next < today;
                (clamped, is_overdue)
            })
            .unwrap_or((0, false));
            let dto = SubscriptionListItemDto {
                id: item.id,
                name: item.name,
                logo: item.logo,
                price: item.price,
                currency_id: item.currency_id,
                next_payment: item.next_payment,
                start_date: item.start_date,
                cycle: item.cycle,
                frequency: item.frequency,
                notes: item.notes,
                payment_method_id: item.payment_method_id,
                payer_user_id: item.payer_user_id,
                category_id: item.category_id,
                notify: item.notify,
                notify_days_before: item.notify_days_before,
                last_notified_date: item.last_notified_date,
                inactive: item.inactive,
                auto_renew: item.auto_renew,
                url: item.url,
                cancellation_date: item.cancellation_date,
                replacement_subscription_id: item.replacement_subscription_id,
                created_at: item.created_at,
                tags: item.tags,
                favorite: item.favorite,
                payment_history: item.payment_history,
                monthly_price: monthly,
                days_left,
                overdue,
                credentials_meta: SubscriptionCredentialsMetaDto::default(),
            };
            Ok(dto)
        })
        .collect::<Result<Vec<_>, crate::errors::AppError>>()?;

    match sort_by.as_str() {
        "name" => rows.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase())),
        "price" => rows.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)),
        _ => rows.sort_by(|a, b| {
            let ad = parse_subscription_date(&a.next_payment);
            let bd = parse_subscription_date(&b.next_payment);
            match (ad, bd) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.next_payment.cmp(&b.next_payment),
            }
        }),
    }

    if req.disabled_to_bottom {
        rows.sort_by_key(|x| x.inactive);
    }

    for row in &mut rows {
        row.credentials_meta = read_meta_index(&guard, &row.id)?;
    }

    Ok(rows)
}

#[tauri::command]
pub fn subscriptions_list(state: State<'_, AppState>) -> Result<Vec<SubscriptionListItemDto>, crate::errors::AppError> {
    list_subscriptions_page(state, None)
}

#[tauri::command]
pub fn subscriptions_next_cycle_date(date: String, cycle: u64, frequency: u64) -> Result<String, crate::errors::AppError> {
    let parsed = parse_subscription_date(&date).ok_or_else(|| "field_invalid_date:date".to_string())?;
    let next = add_cycle_increment(parsed, cycle, frequency, 1).ok_or("failed to calculate next cycle date")?;
    Ok(next.format("%Y-%m-%d").to_string())
}

#[tauri::command]
pub fn subscriptions_payment_dates_in_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<HashMap<String, Vec<u32>>, crate::errors::AppError> {
    if !(1..=12).contains(&month) {
        return Err(crate::errors::AppError::from("month must be between 1 and 12"));
    }

    let subscriptions = {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?
    };

    let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| {
        crate::errors::AppError::from("invalid year/month")
    })?;
    let end = NaiveDate::from_ymd_opt(year, month, last_day_of_month(year, month)).ok_or_else(|| {
        crate::errors::AppError::from("invalid end of month")
    })?;

    let mut result: HashMap<String, Vec<u32>> = HashMap::new();

    for sub in &subscriptions {
        let id = sub.id.as_str();
        let next_payment = match parse_subscription_date(&sub.next_payment) {
            Some(v) => v,
            None => continue,
        };
        let cycle = sub.cycle as u64;
        let frequency = sub.frequency as u64;

        let mut current = match jump_to_window_start(next_payment, cycle, frequency, start, end) {
            Some(v) => v,
            None => {
                result.insert(id.to_string(), Vec::new());
                continue;
            }
        };

        let mut days: Vec<u32> = Vec::new();
        for _ in 0..400 {
            if current >= start && current <= end {
                days.push(current.day());
            }
            if current > end {
                break;
            }
            if let Some(next) = add_cycle_increment(current, cycle, frequency, 1) {
                current = next;
            } else {
                break;
            }
        }

        result.insert(id.to_string(), days);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_overdue_subscriptions(state: State<'_, AppState>) -> Result<Vec<SubscriptionListItemDto>, crate::errors::AppError> {
    let guard = state
        .lock()
        .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let rows = guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?;
    let today = chrono::Local::now().date_naive();
    let mut rows = rows
        .into_iter()
        .filter(|s| {
            !s.inactive
                && parse_subscription_date(&s.next_payment)
                    .map(|d| d < today)
                    .unwrap_or(false)
        })
        .map(|s| {
            Ok(SubscriptionListItemDto {
                id: s.id,
                name: s.name,
                logo: s.logo,
                price: s.price,
                currency_id: s.currency_id,
                next_payment: s.next_payment,
                start_date: s.start_date,
                cycle: s.cycle,
                frequency: s.frequency,
                notes: s.notes,
                payment_method_id: s.payment_method_id,
                payer_user_id: s.payer_user_id,
                category_id: s.category_id,
                notify: s.notify,
                notify_days_before: s.notify_days_before,
                last_notified_date: s.last_notified_date,
                inactive: s.inactive,
                auto_renew: s.auto_renew,
                url: s.url,
                cancellation_date: s.cancellation_date,
                replacement_subscription_id: s.replacement_subscription_id,
                created_at: s.created_at,
                tags: s.tags,
                favorite: s.favorite,
                payment_history: s.payment_history,
                monthly_price: 0.0,
                days_left: 0,
                overdue: true,
                credentials_meta: SubscriptionCredentialsMetaDto::default(),
            })
        })
        .collect::<Result<Vec<_>, crate::errors::AppError>>()?;
    for row in &mut rows {
        row.credentials_meta = read_meta_index(&guard, &row.id)?;
    }
    Ok(rows)
}

#[tauri::command]
pub fn get_upcoming_subscriptions(state: State<'_, AppState>, days: i64, limit: usize) -> Result<Vec<SubscriptionListItemDto>, crate::errors::AppError> {
    let guard = state
        .lock()
        .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let rows = guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?;
    let today = chrono::Local::now().date_naive();
    let end = today + chrono::Duration::days(days);
    let mut rows = rows
        .into_iter()
        .filter(|s| {
            if s.inactive {
                return false;
            }
            parse_subscription_date(&s.next_payment)
                .map(|d| d >= today && d <= end)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        let ad = parse_subscription_date(&a.next_payment);
        let bd = parse_subscription_date(&b.next_payment);
        match (ad, bd) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.next_payment.cmp(&b.next_payment),
        }
    });
    rows.truncate(limit);
    let mut out: Vec<SubscriptionListItemDto> = rows
        .into_iter()
        .map(|s| {
            SubscriptionListItemDto {
                id: s.id,
                name: s.name,
                logo: s.logo,
                price: s.price,
                currency_id: s.currency_id,
                next_payment: s.next_payment,
                start_date: s.start_date,
                cycle: s.cycle,
                frequency: s.frequency,
                notes: s.notes,
                payment_method_id: s.payment_method_id,
                payer_user_id: s.payer_user_id,
                category_id: s.category_id,
                notify: s.notify,
                notify_days_before: s.notify_days_before,
                last_notified_date: s.last_notified_date,
                inactive: s.inactive,
                auto_renew: s.auto_renew,
                url: s.url,
                cancellation_date: s.cancellation_date,
                replacement_subscription_id: s.replacement_subscription_id,
                created_at: s.created_at,
                tags: s.tags,
                favorite: s.favorite,
                payment_history: s.payment_history,
                monthly_price: 0.0,
                days_left: 0,
                overdue: false,
                credentials_meta: SubscriptionCredentialsMetaDto::default(),
            }
        })
        .collect();
    for row in &mut out {
        row.credentials_meta = read_meta_index(&guard, &row.id)?;
    }
    Ok(out)
}

#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub(crate) const WIDGET_UPCOMING_DAYS: i64 = 30;
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub(crate) const WIDGET_UPCOMING_LIMIT: usize = 10;

/// Upcoming subscriptions for the iOS widget snapshot (no credentials, bounded list).
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub(crate) fn collect_upcoming_subscription_docs_for_widget(
    guard: &crate::state::AppStateInner,
) -> Result<Vec<SubscriptionDoc>, crate::errors::AppError> {
    let rows = guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?;
    let today = Local::now().date_naive();
    let end = today + chrono::Duration::days(WIDGET_UPCOMING_DAYS);
    let mut rows = rows
        .into_iter()
        .filter(|s| {
            if s.inactive {
                return false;
            }
            parse_subscription_date(&s.next_payment)
                .map(|d| d >= today && d <= end)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        let ad = parse_subscription_date(&a.next_payment);
        let bd = parse_subscription_date(&b.next_payment);
        match (ad, bd) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.next_payment.cmp(&b.next_payment),
        }
    });
    rows.truncate(WIDGET_UPCOMING_LIMIT);
    Ok(rows)
}

#[tauri::command]
pub fn subscriptions_insert(app: tauri::AppHandle, state: State<'_, AppState>, mut subscription: SubscriptionInputDto) -> Result<(), crate::errors::AppError> {
    subscription.validate()?;
    let creds = subscription.credentials.take();
    let subscription = subscription_row_from_input(subscription)?;
    let sub_id = subscription.id.clone();
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let existing = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &subscription.id)?;
    guard.table_upsert_typed(EntityTable::Subscriptions, &subscription, &subscription.id)?;
    sync_subscription_payment_record_index(&mut guard, existing.as_ref(), Some(&subscription))?;
    credentials_apply_optional(&guard, &sub_id, creds)?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_subscriptions_changed(&app, "insert");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_upsert(app: tauri::AppHandle, state: State<'_, AppState>, mut subscription: SubscriptionInputDto) -> Result<(), crate::errors::AppError> {
    subscription.validate()?;
    let creds = subscription.credentials.take();
    let subscription = subscription_row_from_input(subscription)?;
    let sub_id = subscription.id.clone();
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let existing = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &subscription.id)?;
    guard.table_upsert_typed(EntityTable::Subscriptions, &subscription, &subscription.id)?;
    sync_subscription_payment_record_index(&mut guard, existing.as_ref(), Some(&subscription))?;
    credentials_apply_optional(&guard, &sub_id, creds)?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_subscriptions_changed(&app, "upsert");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_update(app: tauri::AppHandle, state: State<'_, AppState>, mut subscription: SubscriptionInputDto) -> Result<(), crate::errors::AppError> {
    subscription.validate()?;
    let creds = subscription.credentials.take();
    let subscription = subscription_row_from_input(subscription)?;
    let sub_id = subscription.id.clone();
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let existing = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &subscription.id)?;
    guard.table_upsert_typed(EntityTable::Subscriptions, &subscription, &subscription.id)?;
    sync_subscription_payment_record_index(&mut guard, existing.as_ref(), Some(&subscription))?;
    credentials_apply_optional(&guard, &sub_id, creds)?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_subscriptions_changed(&app, "update");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_delete(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let unlinked = unlink_expenses_for_subscription(&mut guard, &id)?;
    if let Some(sub) = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &id)? {
        sync_subscription_payment_record_index(&mut guard, Some(&sub), None)?;
    }
    guard.table_delete_by_id(EntityTable::Subscriptions, &id)?;
    credentials_delete(&guard, &id)?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    if unlinked {
        emit_expenses_changed(&app, "unlink_subscription");
    }
    emit_subscriptions_changed(&app, "delete");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_delete_batch(app: tauri::AppHandle, state: State<'_, AppState>, ids: Vec<String>) -> Result<(), crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let mut any_unlinked = false;
    for id in ids {
        if !id.trim().is_empty() {
            if unlink_expenses_for_subscription(&mut guard, &id)? {
                any_unlinked = true;
            }
            if let Some(sub) = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &id)? {
                sync_subscription_payment_record_index(&mut guard, Some(&sub), None)?;
            }
            guard.table_delete_by_id(EntityTable::Subscriptions, &id)?;
            credentials_delete(&guard, &id)?;
        }
    }
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    if any_unlinked {
        emit_expenses_changed(&app, "unlink_subscription");
    }
    emit_subscriptions_changed(&app, "delete_batch");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_insert_payment_record(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    sub_id: String,
    payment_record: PaymentRecordDto,
) -> Result<(), crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let mut row = guard
        .table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &sub_id)?
        .ok_or_else(|| "subscription not found".to_string())?;
    row.payment_history.insert(0, payment_record.clone());
    let old_row = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &sub_id)?;
    guard.table_upsert_typed(EntityTable::Subscriptions, &row, &row.id)?;
    sync_subscription_payment_record_index(&mut guard, old_row.as_ref(), Some(&row))?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_subscriptions_changed(&app, "insert_payment_record");
    Ok(())
}

#[tauri::command]
pub fn subscriptions_delete_payment_record(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let indexed_sub_id = guard.redb_get(&payment_record_index_key(&id))?;
    if let Some(sub_id) = indexed_sub_id {
        if let Some(mut sub) = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &sub_id)? {
            let before = sub.payment_history.len();
            sub.payment_history.retain(|pr| pr.id != id);
            if sub.payment_history.len() != before {
                let old_sub = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &sub.id)?;
                guard.table_upsert_typed(EntityTable::Subscriptions, &sub, &sub.id)?;
                sync_subscription_payment_record_index(&mut guard, old_sub.as_ref(), Some(&sub))?;
                drop(guard);
                let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
                emit_subscriptions_changed(&app, "delete_payment_record");
                return Ok(());
            }
        }
    }

    // Fallback for legacy/stale index: one scan, then auto-heal index.
    let mut rows: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    for item in &mut rows {
        let before = item.payment_history.len();
        item.payment_history.retain(|pr| pr.id != id);
        if item.payment_history.len() != before {
            let old_item = guard.table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &item.id)?;
            guard.table_upsert_typed(EntityTable::Subscriptions, item, &item.id)?;
            sync_subscription_payment_record_index(&mut guard, old_item.as_ref(), Some(item))?;
            drop(guard);
            let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
            emit_subscriptions_changed(&app, "delete_payment_record");
            return Ok(());
        }
    }
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_subscriptions_changed(&app, "delete_payment_record");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_subscription_payment_record_index_sets_and_clears_keys() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let mut state = crate::state::AppStateInner {
            db: std::sync::Arc::new(db),
            app_data: doc.clone(),
        };

        let mut sub = doc.subscriptions[0].clone();
        sub.id = "sub-index-test".to_string();
        sub.payment_history = vec![
            crate::models::PaymentRecordDto {
                id: "pr-a".to_string(),
                date: "2026-01-01".to_string(),
                amount: 1.0,
                currency_id: "cur-2".to_string(),
                note: String::new(),
            },
            crate::models::PaymentRecordDto {
                id: "pr-b".to_string(),
                date: "2026-01-02".to_string(),
                amount: 2.0,
                currency_id: "cur-2".to_string(),
                note: String::new(),
            },
        ];

        sync_subscription_payment_record_index(&mut state, None, Some(&sub)).expect("set index");
        assert_eq!(
            state.redb_get(&payment_record_index_key("pr-a")).expect("read index"),
            Some("sub-index-test".to_string())
        );
        assert_eq!(
            state.redb_get(&payment_record_index_key("pr-b")).expect("read index"),
            Some("sub-index-test".to_string())
        );

        sync_subscription_payment_record_index(&mut state, Some(&sub), None).expect("clear index");
        assert!(
            state
                .redb_get(&payment_record_index_key("pr-a"))
                .expect("read cleared")
                .is_none()
        );
        assert!(
            state
                .redb_get(&payment_record_index_key("pr-b"))
                .expect("read cleared")
                .is_none()
        );
    }
}
