use chrono::{Datelike, Duration, Local, NaiveDate};
use std::collections::HashMap;
use tauri::Emitter;
use tauri::State;
use crate::AppState;
use crate::models::{
    ExpenseDoc, PaymentRecordDto, SubscriptionCredentialsMetaDto, SubscriptionDoc,
    SubscriptionInputDto, SubscriptionListItemDto, SubscriptionsPageRequestDto,
};
use crate::commands::expenses::{
    apply_expense_payment_record_kv_in_tx, emit_expenses_changed, update_expense_payment_record_index,
};
use crate::commands::subscription_credentials::{
    credentials_apply_optional, credentials_delete, read_meta_index,
};
use crate::state::{
    encode_bin, entity_table_upsert_bin_in_tx, rewrite_expense_day_index_in_tx, run_write_transaction,
    update_expense_day_index, EntityTable,
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
    #[cfg(any(target_os = "ios", target_os = "android"))]
    crate::widget_snapshot::export_widget_snapshot_from_app(app);
}

/// `last_notified_date` is owned by the notification check. The UI round-trips a cached copy
/// that goes stale as soon as a check runs, so saving a subscription must not overwrite it.
fn keep_notify_marker(mut row: SubscriptionDoc, existing: Option<&SubscriptionDoc>) -> SubscriptionDoc {
    if let Some(e) = existing {
        row.last_notified_date = e.last_notified_date.clone();
    }
    row
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
    if let Some(head) = trimmed.get(..10) {
        if let Ok(date) = NaiveDate::parse_from_str(head, "%Y-%m-%d") {
            return Ok(date.format("%Y-%m-%d").to_string());
        }
    }
    Err(crate::errors::AppError::from("field_invalid_date:nextPayment"))
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

/// First cycle occurrence `>= on_or_after`, stepping from `anchor` by whole cycles.
/// Each step is computed from the anchor (not chained), so month-end clamping does not drift.
/// `None` for an unknown cycle or when the target is unreachable within the step cap.
pub(crate) fn first_cycle_date_on_or_after(
    anchor: NaiveDate,
    cycle: u64,
    frequency: u64,
    on_or_after: NaiveDate,
) -> Option<NaiveDate> {
    if anchor >= on_or_after {
        return Some(anchor);
    }
    if !(1..=4).contains(&cycle) {
        return None;
    }
    let freq = frequency.max(1);
    // ponytail: linear scan, ~11 years of daily cycles at most; fine for notification checks.
    for k in 1..=4096_u64 {
        let candidate = add_cycle_increment(anchor, cycle, freq.checked_mul(k)?, 1)?;
        if candidate >= on_or_after {
            return Some(candidate);
        }
    }
    None
}

fn parse_subscription_date(raw: &str) -> Option<NaiveDate> {
    normalize_date_ymd(raw)
        .ok()
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
}

/// Signed days until `next_payment`, elapsed fraction of the current cycle, urgency bucket.
fn schedule_fields(next_payment: &str, cycle: u8, frequency: u32, today: NaiveDate) -> (i64, f32, String) {
    let Some(next) = parse_subscription_date(next_payment) else {
        return (0, 0.0, "normal".to_string());
    };
    let days_left = (next - today).num_days();
    let cycle_progress = match add_cycle_increment(next, cycle as u64, frequency as u64, -1) {
        Some(prev) if prev < next => {
            ((today - prev).num_days() as f32 / (next - prev).num_days() as f32).clamp(0.0, 1.0)
        }
        _ if today >= next => 1.0,
        _ => 0.0,
    };
    let urgency = match days_left {
        d if d < 0 => "overdue",
        0..=3 => "danger",
        4..=7 => "warn",
        _ => "normal",
    };
    (days_left, cycle_progress, urgency.to_string())
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
    list_subscription_rows(&guard, request, Local::now().date_naive())
}

fn list_subscription_rows(
    guard: &crate::state::AppStateInner,
    request: Option<SubscriptionsPageRequestDto>,
    today: NaiveDate,
) -> Result<Vec<SubscriptionListItemDto>, crate::errors::AppError> {
    let subscriptions = guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?;
    let category_names: HashMap<&str, &str> =
        guard.app_data.categories.iter().map(|c| (c.id.as_str(), c.name.as_str())).collect();
    let payment_names: HashMap<&str, &str> =
        guard.app_data.payment_methods.iter().map(|p| (p.id.as_str(), p.name.as_str())).collect();
    let rates = crate::rate_map(&guard.app_data);
    let req = request.unwrap_or_default();
    let search = req.search.trim().to_lowercase();
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
                let haystack = format!(
                    "{} {} {} {} {} {}",
                    item.name,
                    item.notes,
                    item.category_id,
                    item.payment_method_id,
                    category_names.get(item.category_id.as_str()).unwrap_or(&""),
                    payment_names.get(item.payment_method_id.as_str()).unwrap_or(&""),
                )
                .to_lowercase();
                if !haystack.contains(&search) {
                    return false;
                }
            }
            true
        })
        .map(|item| {
            let monthly = crate::price_per_month(item.cycle, item.frequency, item.price);
            let price_main = crate::convert_to_main(item.price, &item.currency_id, &rates);
            let monthly_price_main = crate::convert_to_main(monthly, &item.currency_id, &rates);
            let (days_left, cycle_progress, urgency) =
                schedule_fields(&item.next_payment, item.cycle, item.frequency, today);
            let overdue = !item.inactive && !item.auto_renew && days_left < 0;
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
                price_main,
                monthly_price_main,
                days_left,
                overdue,
                cycle_progress,
                urgency,
                credentials_meta: SubscriptionCredentialsMetaDto::default(),
            };
            Ok(dto)
        })
        .collect::<Result<Vec<_>, crate::errors::AppError>>()?;

    match sort_by.as_str() {
        // Unicode-aware: `to_ascii_lowercase` left Cyrillic etc. case-sensitive.
        "name" => rows.sort_by_key(|a| a.name.to_lowercase()),
        // Compare in the main currency: raw prices across currencies are not comparable.
        "price" => rows.sort_by_key(|a| a.price_main),
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
        row.credentials_meta = read_meta_index(guard, &row.id)?;
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
    let (start, end) = month_bounds(year, month)?;
    let subscriptions = {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        guard.table_list_typed::<SubscriptionDoc>(EntityTable::Subscriptions)?
    };

    let mut result: HashMap<String, Vec<u32>> = HashMap::new();
    for sub in &subscriptions {
        if let Some(dates) = payment_dates_in_window(sub, start, end) {
            result.insert(sub.id.clone(), dates.iter().map(|d| d.day()).collect());
        }
    }
    Ok(result)
}

fn month_bounds(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate), crate::errors::AppError> {
    if !(1..=12).contains(&month) {
        return Err(crate::errors::AppError::from("month must be between 1 and 12"));
    }
    let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| {
        crate::errors::AppError::from("invalid year/month")
    })?;
    let end = NaiveDate::from_ymd_opt(year, month, last_day_of_month(year, month)).ok_or_else(|| {
        crate::errors::AppError::from("invalid end of month")
    })?;
    Ok((start, end))
}

/// Projected payment dates of `sub` inside `[start, end]`; `None` when `next_payment` is unparseable.
pub(crate) fn payment_dates_in_window(sub: &SubscriptionDoc, start: NaiveDate, end: NaiveDate) -> Option<Vec<NaiveDate>> {
    let next_payment = parse_subscription_date(&sub.next_payment)?;
    let cycle = sub.cycle as u64;
    let frequency = sub.frequency as u64;
    let Some(mut current) = jump_to_window_start(next_payment, cycle, frequency, start, end) else {
        return Some(Vec::new());
    };
    let mut dates = Vec::new();
    for _ in 0..400 {
        if current >= start && current <= end {
            dates.push(current);
        }
        if current > end {
            break;
        }
        match add_cycle_increment(current, cycle, frequency, 1) {
            Some(next) => current = next,
            None => break,
        }
    }
    Some(dates)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalendarMonthStatsDto {
    pub(crate) active_subscriptions: usize,
    pub(crate) payments_count: usize,
    #[serde(with = "crate::money::serde_ipc")]
    pub(crate) total_cost: rust_decimal::Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub(crate) amount_due: rust_decimal::Decimal,
}

fn calendar_month_stats_for(
    data: &crate::AppDataDoc,
    year: i32,
    month: u32,
    today: NaiveDate,
) -> Result<CalendarMonthStatsDto, crate::errors::AppError> {
    let (start, end) = month_bounds(year, month)?;
    let rates = crate::rate_map(data);
    let mut stats = CalendarMonthStatsDto {
        active_subscriptions: 0,
        payments_count: 0,
        total_cost: rust_decimal::Decimal::ZERO,
        amount_due: rust_decimal::Decimal::ZERO,
    };
    for sub in data.subscriptions.iter().filter(|s| !s.inactive) {
        let dates = payment_dates_in_window(sub, start, end).unwrap_or_default();
        if dates.is_empty() {
            continue;
        }
        let price = crate::convert_to_main(sub.price, &sub.currency_id, &rates);
        stats.active_subscriptions += 1;
        stats.payments_count += dates.len();
        for d in dates {
            stats.total_cost += price;
            if d >= today {
                stats.amount_due += price;
            }
        }
    }
    Ok(stats)
}

#[tauri::command]
pub fn calendar_month_stats(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<CalendarMonthStatsDto, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    calendar_month_stats_for(&guard.app_data, year, month, Local::now().date_naive())
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
            let (days_left, cycle_progress, urgency) = schedule_fields(&s.next_payment, s.cycle, s.frequency, today);
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
                monthly_price: rust_decimal::Decimal::ZERO,
                price_main: rust_decimal::Decimal::ZERO,
                monthly_price_main: rust_decimal::Decimal::ZERO,
                days_left,
                overdue: true,
                cycle_progress,
                urgency,
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
            let (days_left, cycle_progress, urgency) = schedule_fields(&s.next_payment, s.cycle, s.frequency, today);
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
                monthly_price: rust_decimal::Decimal::ZERO,
                price_main: rust_decimal::Decimal::ZERO,
                monthly_price_main: rust_decimal::Decimal::ZERO,
                days_left,
                overdue: false,
                cycle_progress,
                urgency,
                credentials_meta: SubscriptionCredentialsMetaDto::default(),
            }
        })
        .collect();
    for row in &mut out {
        row.credentials_meta = read_meta_index(&guard, &row.id)?;
    }
    Ok(out)
}

#[cfg_attr(not(any(target_os = "ios", target_os = "android")), allow(dead_code))]
pub(crate) const WIDGET_UPCOMING_DAYS: i64 = 30;
#[cfg_attr(not(any(target_os = "ios", target_os = "android")), allow(dead_code))]
pub(crate) const WIDGET_UPCOMING_LIMIT: usize = 10;

/// Upcoming subscriptions for home screen widget snapshots (no credentials, bounded list).
#[cfg_attr(not(any(target_os = "ios", target_os = "android")), allow(dead_code))]
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
    let subscription = keep_notify_marker(subscription, existing.as_ref());
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
    let subscription = keep_notify_marker(subscription, existing.as_ref());
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
    let subscription = keep_notify_marker(subscription, existing.as_ref());
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
    mut payment_record: PaymentRecordDto,
) -> Result<(), crate::errors::AppError> {
    // Same rules as `subscriptions_record_payment`: a real date, a non-negative amount.
    payment_record.date = normalize_date_ymd(payment_record.date.trim())
        .map_err(|_| crate::errors::AppError::from("field_invalid_date:date"))?;
    if !crate::money::is_valid_non_negative(&payment_record.amount) {
        return Err(crate::errors::AppError::from("field_invalid_number:amount"));
    }
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

/// One locked unit: payment record + next cycle + linked expense + indexes.
fn record_payment_in_guard(
    guard: &mut crate::state::AppStateInner,
    sub_id: &str,
    date: Option<String>,
    amount: Option<String>,
    note: Option<String>,
    today: NaiveDate,
) -> Result<SubscriptionDoc, crate::errors::AppError> {
    let old_row = guard
        .table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, sub_id)?
        .ok_or_else(|| "subscription not found".to_string())?;
    let date = match date.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        Some(raw) => normalize_date_ymd(raw).map_err(|_| crate::errors::AppError::from("field_invalid_date:date"))?,
        None => today.format("%Y-%m-%d").to_string(),
    };
    let amount = match amount.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
        Some(raw) => crate::money::parse_money_str(raw)
            .ok()
            .filter(crate::money::is_valid_non_negative)
            .ok_or_else(|| crate::errors::AppError::from("field_invalid_number:amount"))?,
        None => old_row.price,
    };
    let next = parse_subscription_date(&old_row.next_payment)
        .and_then(|d| add_cycle_increment(d, old_row.cycle as u64, old_row.frequency as u64, 1))
        .ok_or_else(|| crate::errors::AppError::from("field_invalid_date:nextPayment"))?;
    let stamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let note = note.unwrap_or_default();
    let record = PaymentRecordDto {
        id: format!("pr-{stamp}"),
        date: date.clone(),
        amount,
        currency_id: old_row.currency_id.clone(),
        note: note.clone(),
    };

    let mut row = old_row.clone();
    row.payment_history.insert(0, record.clone());
    row.next_payment = next.format("%Y-%m-%d").to_string();
    guard.table_upsert_typed(EntityTable::Subscriptions, &row, &row.id)?;
    sync_subscription_payment_record_index(guard, Some(&old_row), Some(&row))?;

    let expense = ExpenseDoc {
        id: format!("exp-{stamp}"),
        updated_at: chrono::Utc::now().timestamp_millis(),
        name: row.name.clone(),
        amount,
        currency_id: row.currency_id.clone(),
        created_at: crate::models::normalize_expense_timestamp(&date)?,
        category_id: row.category_id.clone(),
        tags: row.tags.clone(),
        payment_method_id: row.payment_method_id.clone(),
        payer_user_id: row.payer_user_id.clone(),
        notes: note,
        url: String::new(),
        subscription_id: row.id.clone(),
        payment_record_id: record.id,
    };
    guard.table_upsert_typed(EntityTable::Expenses, &expense, &expense.id)?;
    update_expense_payment_record_index(guard, None, Some(&expense))?;
    update_expense_day_index(guard.db.as_ref(), None, Some(&expense))?;
    // Re-read: the upsert stamps `updated_at`.
    Ok(guard
        .table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, sub_id)?
        .unwrap_or(row))
}

#[tauri::command]
pub fn subscriptions_record_payment(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    sub_id: String,
    date: Option<String>,
    amount: Option<String>,
    note: Option<String>,
) -> Result<SubscriptionDoc, crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let row = record_payment_in_guard(&mut guard, &sub_id, date, amount, note, Local::now().date_naive())?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    emit_expenses_changed(&app, "record_payment");
    emit_subscriptions_changed(&app, "record_payment");
    Ok(row)
}

/// Removes the payment record from its subscription and deletes the linked expense(s).
/// Returns whether any expense was deleted.
fn delete_payment_record_in_guard(
    guard: &mut crate::state::AppStateInner,
    id: &str,
) -> Result<bool, crate::errors::AppError> {
    if id.trim().is_empty() {
        return Ok(false);
    }
    let has_record = |s: &SubscriptionDoc| s.payment_history.iter().any(|pr| pr.id == id);
    let mut owner = match guard.redb_get(&payment_record_index_key(id))? {
        Some(sub_id) => guard
            .table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, &sub_id)?
            .filter(has_record),
        None => None,
    };
    if owner.is_none() {
        // Fallback for legacy/stale index: one scan, then auto-heal index.
        let rows: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
        owner = rows.into_iter().find(has_record);
    }
    if let Some(old_sub) = owner {
        let mut sub = old_sub.clone();
        sub.payment_history.retain(|pr| pr.id != id);
        guard.table_upsert_typed(EntityTable::Subscriptions, &sub, &sub.id)?;
        sync_subscription_payment_record_index(guard, Some(&old_sub), Some(&sub))?;
    }

    let linked: Vec<ExpenseDoc> = guard
        .app_data
        .expenses
        .iter()
        .filter(|e| e.payment_record_id == id)
        .cloned()
        .collect();
    for expense in &linked {
        guard.table_delete_by_id(EntityTable::Expenses, &expense.id)?;
        update_expense_payment_record_index(guard, Some(expense), None)?;
        update_expense_day_index(guard.db.as_ref(), Some(expense), None)?;
    }
    Ok(!linked.is_empty())
}

#[tauri::command]
pub fn subscriptions_delete_payment_record(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let expenses_deleted = delete_payment_record_in_guard(&mut guard, &id)?;
    drop(guard);
    let _ = crate::commands::notifications::notifications_reschedule_all(app.clone(), state)?;
    if expenses_deleted {
        emit_expenses_changed(&app, "delete_by_payment_record");
    }
    emit_subscriptions_changed(&app, "delete_payment_record");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(raw: &str) -> NaiveDate {
        NaiveDate::parse_from_str(raw, "%Y-%m-%d").unwrap()
    }

    fn persisted_state(doc: crate::AppDataDoc) -> (tempfile::TempDir, crate::state::AppStateInner) {
        let (dir, db) = crate::test_support::temp_db().expect("temp db");
        crate::state::save_app_data_typed(&db, &doc, &crate::test_support::default_config()).expect("save");
        let state = crate::state::AppStateInner { db: std::sync::Arc::new(db), app_data: doc };
        state.redb_set("test:kv-init", "1").expect("create kv table");
        (dir, state)
    }

    #[test]
    fn schedule_fields_signed_days_progress_and_urgency() {
        // Monthly, next 2026-05-11 -> previous cycle 2026-04-11 (30 days).
        let (d, p, u) = schedule_fields("2026-05-11", 3, 1, ymd("2026-05-01"));
        assert_eq!((d, u.as_str()), (10, "normal"));
        assert!((p - 20.0 / 30.0).abs() < 1e-6, "{p}");
        assert_eq!(schedule_fields("2026-05-11", 3, 1, ymd("2026-05-04")).2, "warn");
        assert_eq!(schedule_fields("2026-05-11", 3, 1, ymd("2026-05-08")).2, "danger");
        assert_eq!(schedule_fields("2026-05-11", 3, 1, ymd("2026-05-11")).2, "danger");
        let (d, p, u) = schedule_fields("2026-05-11", 3, 1, ymd("2026-05-13"));
        assert_eq!((d, p, u.as_str()), (-2, 1.0, "overdue"));
        assert_eq!(schedule_fields("bad", 3, 1, ymd("2026-05-13")), (0, 0.0, "normal".to_string()));
    }

    #[test]
    fn list_rows_signed_days_left_and_search_by_catalog_names() {
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let category_name = doc.categories[0].name.clone();
        let payment_name = doc.payment_methods[0].name.clone();
        let (_dir, state) = persisted_state(doc);
        let req = |search: &str| {
            Some(SubscriptionsPageRequestDto { search: search.to_string(), ..Default::default() })
        };
        let today = ymd("2026-05-04");

        let rows = list_subscription_rows(&state, req(""), today).expect("rows");
        let row = rows.iter().find(|r| r.id == "sub-test-1").expect("sub");
        assert_eq!(row.days_left, -3);
        assert_eq!(row.urgency, "overdue");
        assert!(!row.overdue, "auto-renew is never flagged overdue");

        for q in [category_name.to_uppercase(), payment_name.to_lowercase()] {
            let rows = list_subscription_rows(&state, req(&q), today).expect("rows");
            assert!(rows.iter().any(|r| r.id == "sub-test-1"), "search {q}");
        }
        assert!(list_subscription_rows(&state, req("zzz-no-match"), today).expect("rows").is_empty());
    }

    #[test]
    fn list_rows_carry_main_currency_prices() {
        let mut doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        doc.currencies[1].rate = rust_decimal::Decimal::from(2);
        let other = doc.currencies[1].id.clone();
        let sub = doc.subscriptions.iter_mut().find(|s| s.id == "sub-test-1").expect("sub");
        sub.currency_id = other;
        sub.price = rust_decimal::Decimal::from(24);
        sub.cycle = 4; // yearly
        sub.frequency = 1;
        let (_dir, state) = persisted_state(doc);
        let rows = list_subscription_rows(&state, None, ymd("2026-05-04")).expect("rows");
        let row = rows.iter().find(|r| r.id == "sub-test-1").expect("row");
        assert_eq!(row.price_main, rust_decimal::Decimal::from(12));
        assert_eq!(row.monthly_price, rust_decimal::Decimal::from(2));
        assert_eq!(row.monthly_price_main, rust_decimal::Decimal::from(1));
    }

    #[test]
    fn price_sort_compares_in_main_currency() {
        let mut doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        doc.currencies[1].rate = rust_decimal::Decimal::from(100);
        let cheap_foreign = doc.currencies[1].id.clone();
        let base = doc.subscriptions[0].clone();
        doc.subscriptions = vec![
            SubscriptionDoc { id: "foreign".into(), price: rust_decimal::Decimal::from(500), currency_id: cheap_foreign, ..base.clone() },
            SubscriptionDoc { id: "main".into(), price: rust_decimal::Decimal::from(10), ..base },
        ];
        let (_dir, state) = persisted_state(doc);
        let req: SubscriptionsPageRequestDto = serde_json::from_value(serde_json::json!({ "sortBy": "price" })).unwrap();
        let rows = list_subscription_rows(&state, Some(req), ymd("2026-05-04")).expect("rows");
        let ids: Vec<_> = rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["foreign", "main"], "500 at rate 100 = 5 main < 10 main");
    }

    #[test]
    fn calendar_month_stats_counts_active_and_converts_to_main() {
        let mut doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        doc.currencies[1].rate = rust_decimal::Decimal::from(2);
        let main_sub = doc.subscriptions.last_mut().expect("sub");
        main_sub.price = "19.99".parse().unwrap(); // fixture uses lossy from_f64
        let base = main_sub.clone();
        // base: monthly 19.99 in main currency, next 2026-05-01.
        let weekly = SubscriptionDoc {
            id: "sub-weekly".into(),
            price: rust_decimal::Decimal::from(10),
            currency_id: doc.currencies[1].id.clone(),
            cycle: 2,
            next_payment: "2026-05-04".into(),
            ..base.clone()
        };
        let inactive = SubscriptionDoc { id: "sub-off".into(), inactive: true, ..base.clone() };
        let yearly_elsewhere = SubscriptionDoc { id: "sub-year".into(), cycle: 4, next_payment: "2026-09-01".into(), ..base };
        doc.subscriptions.extend([weekly, inactive, yearly_elsewhere]);

        let stats = calendar_month_stats_for(&doc, 2026, 5, ymd("2026-05-18")).expect("stats");
        assert_eq!(stats.active_subscriptions, 2);
        assert_eq!(stats.payments_count, 5, "May 1 + weekly 4/11/18/25");
        assert_eq!(stats.total_cost, "39.99".parse::<rust_decimal::Decimal>().unwrap(), "19.99 + 4 * 10/2");
        assert_eq!(stats.amount_due, rust_decimal::Decimal::from(10), "today (18th) counts as due");
        let json = serde_json::to_value(&stats).unwrap();
        assert_eq!(json["totalCost"], "39.99");
        assert!(calendar_month_stats_for(&doc, 2026, 13, ymd("2026-05-18")).is_err());
    }

    #[test]
    fn record_payment_then_delete_record_removes_linked_expense() {
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let (_dir, mut state) = persisted_state(doc);
        let today = ymd("2026-05-03");

        let bad = record_payment_in_guard(&mut state, "sub-test-1", None, Some("-1".into()), None, today);
        assert_eq!(bad.unwrap_err().to_string(), "field_invalid_number:amount");

        let sub = record_payment_in_guard(&mut state, "sub-test-1", None, None, Some("n".into()), today).expect("record");
        assert_eq!(sub.next_payment, "2026-06-01", "advanced one monthly cycle");
        assert_eq!(sub.payment_history.len(), 2);
        let pr = sub.payment_history[0].clone();
        assert_eq!((pr.date.as_str(), pr.amount, pr.currency_id.as_str()), ("2026-05-03", sub.price, sub.currency_id.as_str()));
        assert_eq!(
            state.redb_get(&payment_record_index_key(&pr.id)).expect("idx"),
            Some("sub-test-1".to_string())
        );
        let exp = state
            .app_data
            .expenses
            .iter()
            .find(|e| e.payment_record_id == pr.id)
            .cloned()
            .expect("linked expense");
        assert_eq!(exp.subscription_id, "sub-test-1");
        assert_eq!((&exp.category_id, &exp.tags, exp.amount), (&sub.category_id, &sub.tags, sub.price));
        assert!(exp.created_at.starts_with("2026-05-03T12:00:00"));
        let day = crate::state::expense_ids_in_day_range(state.db.as_ref(), today, today).expect("day idx");
        assert!(day.contains(&exp.id));

        // Empty id must not wipe unlinked expenses (their payment_record_id is "").
        let before = state.app_data.expenses.len();
        assert!(!delete_payment_record_in_guard(&mut state, " ").expect("noop"));
        assert_eq!(state.app_data.expenses.len(), before);

        assert!(delete_payment_record_in_guard(&mut state, &pr.id).expect("delete"));
        let sub = state
            .table_get_by_id_typed::<SubscriptionDoc>(EntityTable::Subscriptions, "sub-test-1")
            .expect("read")
            .expect("sub");
        assert!(sub.payment_history.iter().all(|r| r.id != pr.id));
        assert!(state.table_get_expense_by_id(&exp.id).expect("read").is_none());
        assert!(state.app_data.expenses.iter().all(|e| e.id != exp.id));
        assert!(state.redb_get(&payment_record_index_key(&pr.id)).expect("idx").is_none());
        let day = crate::state::expense_ids_in_day_range(state.db.as_ref(), today, today).expect("day idx");
        assert!(!day.contains(&exp.id));
    }

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
                amount: crate::money::from_f64_legacy(1.0),
                currency_id: "cur-2".to_string(),
                note: String::new(),
            },
            crate::models::PaymentRecordDto {
                id: "pr-b".to_string(),
                date: "2026-01-02".to_string(),
                amount: crate::money::from_f64_legacy(2.0),
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
