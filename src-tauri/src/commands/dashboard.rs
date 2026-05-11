use chrono::Datelike;
use tauri::State;
use crate::AppState;

fn parse_date_ymd(raw: &str) -> Option<chrono::NaiveDate> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(date);
    }
    if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Some(date_time.date_naive());
    }
    if trimmed.len() >= 10 {
        return chrono::NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").ok();
    }
    None
}

fn expense_in_calendar_month(e: &crate::models::ExpenseDoc, year: i32, month: u32) -> bool {
    e.naive_date()
        .map(|d| d.year() == year && d.month() == month)
        .unwrap_or(false)
}

fn expense_in_year(e: &crate::models::ExpenseDoc, year: i32) -> bool {
    e.naive_date().map(|d| d.year() == year).unwrap_or(false)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };
    (next - first).num_days() as u32
}

/// Current calendar month from the 1st through `today` (month-to-date).
fn is_expense_mtd_current_doc(e: &crate::models::ExpenseDoc, today: chrono::NaiveDate) -> bool {
    let Some(d) = e.naive_date() else {
        return false;
    };
    let start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    d >= start && d <= today
}

/// Previous month, same calendar span: 1st through the same day-of-month as today
/// (capped to the last day of that month — e.g. May 31 vs Apr 1–30).
fn is_expense_mtd_previous_doc(e: &crate::models::ExpenseDoc, today: chrono::NaiveDate) -> bool {
    let Some(d) = e.naive_date() else {
        return false;
    };
    let (py, pm) = calendar_previous_ym(today);
    let start = chrono::NaiveDate::from_ymd_opt(py, pm, 1).unwrap();
    let dim = days_in_month(py, pm);
    let end_day = today.day().min(dim);
    let end = chrono::NaiveDate::from_ymd_opt(py, pm, end_day).unwrap();
    d >= start && d <= end
}

fn calendar_previous_ym(today: chrono::NaiveDate) -> (i32, u32) {
    if today.month() == 1 {
        (today.year() - 1, 12)
    } else {
        (today.year(), today.month() - 1)
    }
}

fn ym_label(year: i32, month: u32) -> String {
    format!("{}-{:02}", year, month)
}

fn shift_calendar_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let mut y = year as i64;
    let mut m = month as i64 + delta as i64;
    while m > 12 {
        m -= 12;
        y += 1;
    }
    while m < 1 {
        m += 12;
        y -= 1;
    }
    (y as i32, m as u32)
}

fn is_last_day_of_month(d: chrono::NaiveDate) -> bool {
    d.day() == days_in_month(d.year(), d.month())
}

fn month_expense_totals(
    expenses: &[crate::models::ExpenseDoc],
    rates: &std::collections::HashMap<String, f64>,
    year: i32,
    month: u32,
) -> (f64, usize) {
    let total: f64 = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, year, month))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    let count = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, year, month))
        .count();
    (total, count)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MonthComparisonStyle {
    Mtd,
    /// Two fully completed months: M−1 vs M−2 (when the current month is still in progress and MTD is empty).
    CompletedPair,
    /// Full calendar month totals: current vs previous (e.g. month-end or when M−1/M−2 pair has no data).
    FullMonths,
}

struct ResolvedMonthComparison {
    current_total: f64,
    previous_total: f64,
    current_count: usize,
    previous_count: usize,
    style: MonthComparisonStyle,
    current_month_label: String,
    previous_month_label: String,
}

fn comparison_style_str(s: MonthComparisonStyle) -> &'static str {
    match s {
        MonthComparisonStyle::Mtd => "mtd",
        MonthComparisonStyle::CompletedPair => "completedPair",
        MonthComparisonStyle::FullMonths => "fullMonths",
    }
}

/// Month-to-date totals vs same calendar span in the previous month.
fn mtd_comparison_totals(
    expenses: &[crate::models::ExpenseDoc],
    rates: &std::collections::HashMap<String, f64>,
    today: chrono::NaiveDate,
) -> (f64, f64) {
    let current: f64 = expenses
        .iter()
        .filter(|e| is_expense_mtd_current_doc(e, today))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    let previous: f64 = expenses
        .iter()
        .filter(|e| is_expense_mtd_previous_doc(e, today))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    (current, previous)
}

fn mtd_comparison_counts(expenses: &[crate::models::ExpenseDoc], today: chrono::NaiveDate) -> (usize, usize) {
    let current = expenses.iter().filter(|e| is_expense_mtd_current_doc(e, today)).count();
    let previous = expenses.iter().filter(|e| is_expense_mtd_previous_doc(e, today)).count();
    (current, previous)
}

/// Full calendar months: current month vs previous month (all recorded days in each).
fn full_month_comparison_totals(
    expenses: &[crate::models::ExpenseDoc],
    rates: &std::collections::HashMap<String, f64>,
    today: chrono::NaiveDate,
) -> (f64, f64) {
    let (py, pm) = calendar_previous_ym(today);
    let current: f64 = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    let previous: f64 = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, py, pm))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    (current, previous)
}

fn full_month_comparison_counts(expenses: &[crate::models::ExpenseDoc], today: chrono::NaiveDate) -> (usize, usize) {
    let (py, pm) = calendar_previous_ym(today);
    let current = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .count();
    let previous = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, py, pm))
        .count();
    (current, previous)
}

/// MTD first; if MTD is empty and we are **not** on the last day of the month, compare two completed months (M−1 vs M−2).
/// Otherwise fall back to full calendar months (current vs previous), then zeros with MTD labels.
fn resolve_month_comparison(
    expenses: &[crate::models::ExpenseDoc],
    rates: &std::collections::HashMap<String, f64>,
    today: chrono::NaiveDate,
) -> ResolvedMonthComparison {
    let (mtd_c, mtd_p) = mtd_comparison_totals(expenses, rates, today);
    let (cc_mtd, pc_mtd) = mtd_comparison_counts(expenses, today);

    let cur_label = ym_label(today.year(), today.month());
    let (py0, pm0) = calendar_previous_ym(today);
    let prev_label = ym_label(py0, pm0);

    if mtd_c != 0.0 || mtd_p != 0.0 {
        return ResolvedMonthComparison {
            current_total: mtd_c,
            previous_total: mtd_p,
            current_count: cc_mtd,
            previous_count: pc_mtd,
            style: MonthComparisonStyle::Mtd,
            current_month_label: cur_label,
            previous_month_label: prev_label,
        };
    }

    let (y_m1, m_m1) = calendar_previous_ym(today);
    let anchor_m1 = chrono::NaiveDate::from_ymd_opt(y_m1, m_m1, 1).unwrap();
    let (y_m2, m_m2) = calendar_previous_ym(anchor_m1);
    let (pair_c, pair_cc) = month_expense_totals(expenses, rates, y_m1, m_m1);
    let (pair_p, pair_pc) = month_expense_totals(expenses, rates, y_m2, m_m2);

    if !is_last_day_of_month(today) && (pair_c > 0.0 || pair_p > 0.0) {
        return ResolvedMonthComparison {
            current_total: pair_c,
            previous_total: pair_p,
            current_count: pair_cc,
            previous_count: pair_pc,
            style: MonthComparisonStyle::CompletedPair,
            current_month_label: ym_label(y_m1, m_m1),
            previous_month_label: ym_label(y_m2, m_m2),
        };
    }

    let (full_c, full_p) = full_month_comparison_totals(expenses, rates, today);
    let (fc_c, fc_p) = full_month_comparison_counts(expenses, today);
    if full_c > 0.0 || full_p > 0.0 {
        return ResolvedMonthComparison {
            current_total: full_c,
            previous_total: full_p,
            current_count: fc_c,
            previous_count: fc_p,
            style: MonthComparisonStyle::FullMonths,
            current_month_label: cur_label,
            previous_month_label: prev_label,
        };
    }

    ResolvedMonthComparison {
        current_total: 0.0,
        previous_total: 0.0,
        current_count: 0,
        previous_count: 0,
        style: MonthComparisonStyle::Mtd,
        current_month_label: cur_label,
        previous_month_label: prev_label,
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMostExpensiveDto {
    pub name: String,
    pub price: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardExpenseAggregationDto {
    pub month_total: f64,
    pub year_total: f64,
    pub recent_expenses: Vec<crate::models::ExpenseDoc>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryDto {
    pub has_subscriptions: bool,
    pub active_count: usize,
    pub inactive_count: usize,
    pub total_monthly: f64,
    pub total_yearly: f64,
    pub avg_monthly: f64,
    pub amount_due_this_month: f64,
    pub most_expensive: Option<DashboardMostExpensiveDto>,
    pub budget: f64,
    pub budget_used: Option<f64>,
    pub budget_left: Option<f64>,
    pub over_budget: Option<f64>,
    pub total_savings_monthly: f64,
    pub monthly_expenses_total: f64,
    pub overdue_subscriptions: Vec<crate::models::SubscriptionDoc>,
    pub upcoming_subscriptions: Vec<crate::models::SubscriptionDoc>,
    pub expense_aggregation: DashboardExpenseAggregationDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCostByIdDto {
    pub id: String,
    pub name: String,
    pub cost: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCountByIdDto {
    pub id: String,
    pub name: String,
    pub count: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardChartsDto {
    pub category_costs: Vec<DashboardCostByIdDto>,
    pub pm_counts: Vec<DashboardCountByIdDto>,
    pub member_costs: Vec<DashboardCostByIdDto>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardForecastInnerDto {
    pub next_month: f64,
    pub next_quarter: f64,
    pub next_month_label: String,
    pub quarter_labels: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMonthComparisonDto {
    pub current_month: String,
    pub previous_month: String,
    pub current: f64,
    pub previous: f64,
    pub diff: f64,
    pub diff_percent: f64,
    /// True when comparison is not MTD (`completedPair` or `fullMonths`).
    pub used_full_month_fallback: bool,
    /// `mtd` | `completedPair` | `fullMonths`
    pub comparison_style: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardForecastDto {
    pub forecast: DashboardForecastInnerDto,
    pub month_comparison: DashboardMonthComparisonDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardDayOfWeekStatDto {
    pub day_of_week: usize,
    pub total: f64,
    pub count: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTagExpenseStatDto {
    pub tag: String,
    pub total: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSpendingHistoryDto {
    pub label: String,
    pub year: i32,
    pub month: u32,
    pub amount: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLifetimeCostDto {
    pub subscription_id: String,
    pub name: String,
    pub logo: String,
    pub start_date: String,
    pub months_active: i64,
    pub total_paid: f64,
    pub monthly_equivalent: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCategoryAverageDto {
    pub category_id: String,
    pub category_name: String,
    pub total_monthly: f64,
    pub subscription_count: i64,
    pub average_monthly: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAvgExpenseStatsDto {
    pub avg_amount: f64,
    pub count: usize,
    pub total: f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMonthComparisonDataDto {
    pub current_total: f64,
    pub current_count: usize,
    pub previous_total: f64,
    pub previous_count: usize,
    pub used_full_month_fallback: bool,
    pub comparison_style: String,
    pub current_month_label: String,
    pub previous_month_label: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTrendsDto {
    pub spending_history: Vec<DashboardSpendingHistoryDto>,
    pub lifetime_costs: Vec<DashboardLifetimeCostDto>,
    pub category_averages: Vec<DashboardCategoryAverageDto>,
    pub top_expenses: Vec<crate::models::ExpenseDoc>,
    pub avg_expense_stats: DashboardAvgExpenseStatsDto,
    pub day_of_week_stats: Vec<DashboardDayOfWeekStatDto>,
    pub month_comparison_data: DashboardMonthComparisonDataDto,
    pub tag_expense_stats: Vec<DashboardTagExpenseStatDto>,
}

#[tauri::command]
pub fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummaryDto, crate::errors::AppError> {
    let data = { let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?; guard.doc()? };
    let rates = crate::rate_map(&data);
    let today = chrono::Local::now().date_naive();

    let active = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();
    let inactive_count = data.subscriptions.iter().filter(|s| s.inactive).count();

    let total_monthly: f64 = active
        .iter()
        .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
        .sum();
    let total_yearly = total_monthly * 12.0;
    let avg_monthly = if active.is_empty() { 0.0 } else { total_monthly / active.len() as f64 };

    let amount_due_this_month: f64 = active
        .iter()
        .filter_map(|s| chrono::NaiveDate::parse_from_str(&s.next_payment, "%Y-%m-%d").ok().map(|d| (s, d)))
        .filter(|(_, d)| d.year() == today.year() && d.month() == today.month() && *d >= today)
        .map(|(s, _)| crate::convert_to_main(s.price, &s.currency_id, &rates))
        .sum();

    let most_expensive = active
        .iter()
        .map(|s| {
            let m = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
            DashboardMostExpensiveDto {
                name: s.name.clone(),
                price: m,
            }
        })
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

    let monthly_expenses_total: f64 = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
        .sum();
    let total_spending = total_monthly + monthly_expenses_total;
    let budget = data.settings.budget;
    let budget_used = if budget > 0.0 { Some((total_spending / budget * 100.0).min(100.0)) } else { None };
    let budget_left = if budget > 0.0 { Some((budget - total_spending).max(0.0)) } else { None };
    let over_budget = if budget > 0.0 && total_spending > budget { Some(total_spending - budget) } else { None };

    let total_savings_monthly: f64 = data
        .subscriptions
        .iter()
        .filter(|s| s.inactive)
        .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
        .sum();

    let overdue_subscriptions = active
        .iter()
        .copied()
        .filter(|s| parse_date_ymd(&s.next_payment).map(|d| d < today).unwrap_or(false))
        .collect::<Vec<_>>();
    let upcoming_subscriptions = active
        .iter()
        .copied()
        .filter_map(|s| chrono::NaiveDate::parse_from_str(&s.next_payment, "%Y-%m-%d").ok().map(|d| (s, d)))
        .filter(|(_, d)| *d >= today && *d <= today + chrono::Duration::days(30))
        .map(|(s, _)| s)
        .take(5)
        .collect::<Vec<_>>();

    let month_total: f64 = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| e.amount)
        .sum();
    let year_total: f64 = data
        .expenses
        .iter()
        .filter(|e| expense_in_year(e, today.year()))
        .map(|e| e.amount)
        .sum();
    let mut recent_expenses = data.expenses.clone();
    recent_expenses.sort_by(|a, b| {
        let ad = a.naive_date();
        let bd = b.naive_date();
        match (ad, bd) {
            (Some(x), Some(y)) => y.cmp(&x),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        }
    });
    recent_expenses.truncate(5);

    Ok(DashboardSummaryDto {
        has_subscriptions: !data.subscriptions.is_empty(),
        active_count: active.len(),
        inactive_count,
        total_monthly,
        total_yearly,
        avg_monthly,
        amount_due_this_month,
        most_expensive,
        budget,
        budget_used,
        budget_left,
        over_budget,
        total_savings_monthly,
        monthly_expenses_total,
        overdue_subscriptions: overdue_subscriptions.into_iter().cloned().collect(),
        upcoming_subscriptions: upcoming_subscriptions.into_iter().cloned().collect(),
        expense_aggregation: DashboardExpenseAggregationDto {
            month_total,
            year_total,
            recent_expenses,
        },
    })
}

#[tauri::command]
pub fn get_dashboard_charts(state: State<'_, AppState>) -> Result<DashboardChartsDto, crate::errors::AppError> {
    let data = { let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?; guard.doc()? };
    let rates = crate::rate_map(&data);
    let active = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();

    let category_name = data.categories.iter().map(|c| (c.id.clone(), c.name.clone())).collect::<std::collections::HashMap<_, _>>();
    let payment_name = data.payment_methods.iter().map(|p| (p.id.clone(), p.name.clone())).collect::<std::collections::HashMap<_, _>>();
    let member_name = data.household.iter().map(|h| (h.id.clone(), h.name.clone())).collect::<std::collections::HashMap<_, _>>();

    let mut category_costs: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut pm_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut member_costs: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for s in active {
        let monthly = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
        *category_costs.entry(s.category_id.clone()).or_insert(0.0) += monthly;
        *pm_counts.entry(s.payment_method_id.clone()).or_insert(0) += 1;
        *member_costs.entry(s.payer_user_id.clone()).or_insert(0.0) += monthly;
    }

    let category_costs = category_costs
        .into_iter()
        .map(|(id, cost)| DashboardCostByIdDto {
            name: category_name.get(&id).cloned().unwrap_or_else(|| "Other".into()),
            id,
            cost,
        })
        .collect::<Vec<_>>();
    let pm_counts = pm_counts
        .into_iter()
        .map(|(id, count)| DashboardCountByIdDto {
            name: payment_name.get(&id).cloned().unwrap_or_else(|| "Other".into()),
            id,
            count,
        })
        .collect::<Vec<_>>();
    let member_costs = member_costs
        .into_iter()
        .map(|(id, cost)| DashboardCostByIdDto {
            name: member_name.get(&id).cloned().unwrap_or_else(|| "Other".into()),
            id,
            cost,
        })
        .collect::<Vec<_>>();

    Ok(DashboardChartsDto {
        category_costs,
        pm_counts,
        member_costs,
    })
}

#[tauri::command]
pub fn get_dashboard_forecast(state: State<'_, AppState>) -> Result<DashboardForecastDto, crate::errors::AppError> {
    let data = { let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?; guard.doc()? };
    let rates = crate::rate_map(&data);
    let active = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();
    let next_month: f64 = active
        .iter()
        .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
        .sum();
    let next_quarter = next_month * 3.0;
    let now = chrono::Local::now().date_naive();
    let resolved = resolve_month_comparison(&data.expenses, &rates, now);
    let diff = resolved.current_total - resolved.previous_total;
    let diff_percent = if resolved.previous_total > 0.0 {
        diff / resolved.previous_total * 100.0
    } else if resolved.current_total > 0.0 {
        100.0
    } else {
        0.0
    };
    Ok(DashboardForecastDto {
        forecast: DashboardForecastInnerDto {
            next_month,
            next_quarter,
            next_month_label: String::new(),
            quarter_labels: Vec::new(),
        },
        month_comparison: DashboardMonthComparisonDto {
            current_month: resolved.current_month_label.clone(),
            previous_month: resolved.previous_month_label.clone(),
            current: resolved.current_total,
            previous: resolved.previous_total,
            diff,
            diff_percent,
            used_full_month_fallback: resolved.style != MonthComparisonStyle::Mtd,
            comparison_style: comparison_style_str(resolved.style).to_string(),
        },
    })
}

#[tauri::command]
pub fn get_dashboard_trends(state: State<'_, AppState>) -> Result<DashboardTrendsDto, crate::errors::AppError> {
    let data = { let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?; guard.doc()? };
    let rates = crate::rate_map(&data);
    let now = chrono::Local::now().date_naive();

    let mut top = data.expenses.clone();
    top.retain(|e| expense_in_calendar_month(e, now.year(), now.month()));
    top.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));
    top.truncate(5);

    let month_expenses = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, now.year(), now.month()))
        .collect::<Vec<_>>();
    let total: f64 = month_expenses.iter().map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates)).sum();
    let count = month_expenses.len() as f64;
    let avg = if count > 0.0 { total / count } else { 0.0 };

    let mut day_map = [0.0_f64; 7];
    let mut day_count = [0_i64; 7];
    for e in &month_expenses {
        if let Some(d) = e.naive_date() {
            let idx = d.weekday().num_days_from_sunday() as usize;
            day_map[idx] += e.amount;
            day_count[idx] += 1;
        }
    }
    let day_of_week = (0..7)
        .map(|i| DashboardDayOfWeekStatDto {
            day_of_week: i,
            total: day_map[i],
            count: day_count[i],
        })
        .collect::<Vec<_>>();

    let cmp_res = resolve_month_comparison(&data.expenses, &rates, now);
    let month_expense_count = month_expenses.len();

    let mut tag_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for e in month_expenses {
        for tag in &e.tags {
            *tag_map.entry(tag.clone()).or_insert(0.0) += e.amount;
        }
    }
    let mut tags = tag_map
        .into_iter()
        .map(|(tag, total)| DashboardTagExpenseStatDto { tag, total })
        .collect::<Vec<_>>();
    tags.sort_by(|a, b| {
        b.total
            .partial_cmp(&a.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let active_subs = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();
    let monthly_subs_total: f64 = active_subs.iter().map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates))).sum();
    let mut spending_history = Vec::new();
    for back in (0..6).rev() {
        let (y, m) = shift_calendar_month(now.year(), now.month(), -(back as i32));
        let anchor = chrono::NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let expenses_total: f64 = data
            .expenses
            .iter()
            .filter(|e| expense_in_calendar_month(e, y, m))
            .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
            .sum();
        spending_history.push(DashboardSpendingHistoryDto {
            label: ym_label(y, m),
            year: y,
            month: anchor.month0(),
            amount: monthly_subs_total + expenses_total,
        });
    }

    let lifetime_costs = data.subscriptions.iter().map(|s| {
        let start = parse_date_ymd(&s.start_date).unwrap_or(now);
        let end = s.cancellation_date
            .as_ref()
            .and_then(|x| parse_date_ymd(x))
            .unwrap_or(now);
        let diff_days = (end - start).num_days().max(1) as f64;
        let months_active = (diff_days / 30.44).round().max(1.0) as i64;
        let monthly = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
        DashboardLifetimeCostDto {
            subscription_id: s.id.clone(),
            name: s.name.clone(),
            logo: s.logo.clone(),
            start_date: s.start_date.clone(),
            months_active,
            total_paid: monthly * months_active as f64,
            monthly_equivalent: monthly,
        }
    }).collect::<Vec<_>>();

    let mut cat_map: std::collections::HashMap<String, (f64, i64)> = std::collections::HashMap::new();
    for s in &active_subs {
        let monthly = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
        let item = cat_map.entry(s.category_id.clone()).or_insert((0.0, 0));
        item.0 += monthly;
        item.1 += 1;
    }
    let cat_name = data.categories.iter().map(|c| (c.id.clone(), c.name.clone())).collect::<std::collections::HashMap<_, _>>();
    let category_averages = cat_map
        .into_iter()
        .map(|(id, (total_monthly, count))| DashboardCategoryAverageDto {
            category_name: cat_name.get(&id).cloned().unwrap_or_else(|| "Other".into()),
            category_id: id,
            total_monthly,
            subscription_count: count,
            average_monthly: if count > 0 {
                total_monthly / count as f64
            } else {
                0.0
            },
        })
        .collect::<Vec<_>>();

    Ok(DashboardTrendsDto {
        spending_history,
        lifetime_costs,
        category_averages,
        top_expenses: top,
        avg_expense_stats: DashboardAvgExpenseStatsDto {
            avg_amount: avg,
            count: month_expense_count,
            total,
        },
        day_of_week_stats: day_of_week,
        month_comparison_data: DashboardMonthComparisonDataDto {
            current_total: cmp_res.current_total,
            current_count: cmp_res.current_count,
            previous_total: cmp_res.previous_total,
            previous_count: cmp_res.previous_count,
            used_full_month_fallback: cmp_res.style != MonthComparisonStyle::Mtd,
            comparison_style: comparison_style_str(cmp_res.style).to_string(),
            current_month_label: cmp_res.current_month_label,
            previous_month_label: cmp_res.previous_month_label,
        },
        tag_expense_stats: tags,
    })
}

#[cfg(test)]
mod comparison_tests {
    use super::*;
    use crate::models::ExpenseDoc;

    fn expense_row(y: i32, m: u32, d: u32, amount: f64, currency_id: &str) -> ExpenseDoc {
        ExpenseDoc {
            id: format!("id-{y}-{m}-{d}"),
            updated_at: 1,
            name: "t".into(),
            amount,
            currency_id: currency_id.into(),
            created_at: crate::models::ymd_to_utc_noon_rfc3339(y, m, d).unwrap_or_default(),
            category_id: String::new(),
            tags: vec![],
            payment_method_id: String::new(),
            payer_user_id: String::new(),
            notes: String::new(),
            url: String::new(),
            subscription_id: String::new(),
            payment_record_id: String::new(),
        }
    }

    #[test]
    fn resolve_comparison_prefers_mtd_when_any_bucket_nonzero() {
        let rates = std::collections::HashMap::from([("USD".into(), 1.0)]);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        let expenses = vec![
            expense_row(2026, 5, 3, 10.0, "USD"),
            expense_row(2026, 4, 3, 20.0, "USD"),
        ];
        let r = resolve_month_comparison(&expenses, &rates, today);
        assert_eq!(r.style, MonthComparisonStyle::Mtd, "MTD should win");
        assert_eq!(r.current_total, 10.0);
        assert_eq!(r.previous_total, 20.0);
        assert_eq!(r.current_count, 1);
        assert_eq!(r.previous_count, 1);
    }

    #[test]
    fn resolve_comparison_uses_completed_pair_when_mtd_empty_early_in_month() {
        let rates = std::collections::HashMap::from([("USD".into(), 1.0)]);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        let expenses = vec![
            expense_row(2026, 5, 15, 100.0, "USD"),
            expense_row(2026, 4, 20, 200.0, "USD"),
        ];
        let r = resolve_month_comparison(&expenses, &rates, today);
        assert_eq!(r.style, MonthComparisonStyle::CompletedPair);
        assert_eq!(r.current_total, 200.0, "April M-1");
        assert_eq!(r.previous_total, 0.0, "March M-2");
        assert_eq!(r.current_month_label, "2026-04");
        assert_eq!(r.previous_month_label, "2026-03");
        assert_eq!(r.current_count, 1);
        assert_eq!(r.previous_count, 0);
    }

}
