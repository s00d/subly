use chrono::Datelike;
use rust_decimal::Decimal;
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
    if let Some(head) = trimmed.get(..10) {
        return chrono::NaiveDate::parse_from_str(head, "%Y-%m-%d").ok();
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

/// `YYYY-MM` labels for the `count` calendar months after `today`'s month.
fn next_month_labels(today: chrono::NaiveDate, count: i32) -> Vec<String> {
    (1..=count)
        .map(|k| {
            let (y, m) = shift_calendar_month(today.year(), today.month(), k);
            ym_label(y, m)
        })
        .collect()
}

fn is_last_day_of_month(d: chrono::NaiveDate) -> bool {
    d.day() == days_in_month(d.year(), d.month())
}

fn month_expense_totals(
    expenses: &[crate::models::ExpenseDoc],
    rates: &std::collections::HashMap<String, Decimal>,
    year: i32,
    month: u32,
) -> (Decimal, usize) {
    let total: Decimal = expenses
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
    current_total: Decimal,
    previous_total: Decimal,
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
    rates: &std::collections::HashMap<String, Decimal>,
    today: chrono::NaiveDate,
) -> (Decimal, Decimal) {
    let current: Decimal = expenses
        .iter()
        .filter(|e| is_expense_mtd_current_doc(e, today))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    let previous: Decimal = expenses
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
    rates: &std::collections::HashMap<String, Decimal>,
    today: chrono::NaiveDate,
) -> (Decimal, Decimal) {
    let (py, pm) = calendar_previous_ym(today);
    let current: Decimal = expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, rates))
        .sum();
    let previous: Decimal = expenses
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
    rates: &std::collections::HashMap<String, Decimal>,
    today: chrono::NaiveDate,
) -> ResolvedMonthComparison {
    let (mtd_c, mtd_p) = mtd_comparison_totals(expenses, rates, today);
    let (cc_mtd, pc_mtd) = mtd_comparison_counts(expenses, today);

    let cur_label = ym_label(today.year(), today.month());
    let (py0, pm0) = calendar_previous_ym(today);
    let prev_label = ym_label(py0, pm0);

    if mtd_c != Decimal::ZERO || mtd_p != Decimal::ZERO {
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

    if !is_last_day_of_month(today) && (pair_c > Decimal::ZERO || pair_p > Decimal::ZERO) {
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
    if full_c > Decimal::ZERO || full_p > Decimal::ZERO {
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
        current_total: Decimal::ZERO,
        previous_total: Decimal::ZERO,
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
    #[serde(with = "crate::money::serde_ipc")]
    pub price: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardExpenseAggregationDto {
    #[serde(with = "crate::money::serde_ipc")]
    pub month_total: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub year_total: Decimal,
    pub recent_expenses: Vec<crate::models::ExpenseDoc>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryDto {
    pub has_subscriptions: bool,
    pub active_count: usize,
    pub inactive_count: usize,
    #[serde(with = "crate::money::serde_ipc")]
    pub total_monthly: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub total_yearly: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub avg_monthly: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub amount_due_this_month: Decimal,
    pub most_expensive: Option<DashboardMostExpensiveDto>,
    #[serde(with = "crate::money::serde_ipc")]
    pub budget: Decimal,
    #[serde(with = "crate::money::serde_ipc_option")]
    pub budget_used: Option<Decimal>,
    #[serde(with = "crate::money::serde_ipc_option")]
    pub budget_left: Option<Decimal>,
    #[serde(with = "crate::money::serde_ipc_option")]
    pub over_budget: Option<Decimal>,
    #[serde(with = "crate::money::serde_ipc")]
    pub total_savings_monthly: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub monthly_expenses_total: Decimal,
    pub overdue_subscriptions: Vec<crate::models::SubscriptionDoc>,
    pub upcoming_subscriptions: Vec<crate::models::SubscriptionDoc>,
    pub expense_aggregation: DashboardExpenseAggregationDto,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCostByIdDto {
    pub id: String,
    pub name: String,
    #[serde(with = "crate::money::serde_ipc")]
    pub cost: Decimal,
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
    #[serde(with = "crate::money::serde_ipc")]
    pub next_month: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub next_quarter: Decimal,
    pub next_month_label: String,
    pub quarter_labels: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMonthComparisonDto {
    pub current_month: String,
    pub previous_month: String,
    #[serde(with = "crate::money::serde_ipc")]
    pub current: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub previous: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub diff: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub diff_percent: Decimal,
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
    #[serde(with = "crate::money::serde_ipc")]
    pub total: Decimal,
    pub count: i64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTagExpenseStatDto {
    pub tag: String,
    #[serde(with = "crate::money::serde_ipc")]
    pub total: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSpendingHistoryDto {
    pub label: String,
    pub year: i32,
    pub month: u32,
    #[serde(with = "crate::money::serde_ipc")]
    pub amount: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLifetimeCostDto {
    pub subscription_id: String,
    pub name: String,
    pub logo: String,
    pub start_date: String,
    pub months_active: i64,
    #[serde(with = "crate::money::serde_ipc")]
    pub total_paid: Decimal,
    #[serde(with = "crate::money::serde_ipc")]
    pub monthly_equivalent: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCategoryAverageDto {
    pub category_id: String,
    pub category_name: String,
    #[serde(with = "crate::money::serde_ipc")]
    pub total_monthly: Decimal,
    pub subscription_count: i64,
    #[serde(with = "crate::money::serde_ipc")]
    pub average_monthly: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAvgExpenseStatsDto {
    #[serde(with = "crate::money::serde_ipc")]
    pub avg_amount: Decimal,
    pub count: usize,
    #[serde(with = "crate::money::serde_ipc")]
    pub total: Decimal,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMonthComparisonDataDto {
    #[serde(with = "crate::money::serde_ipc")]
    pub current_total: Decimal,
    pub current_count: usize,
    #[serde(with = "crate::money::serde_ipc")]
    pub previous_total: Decimal,
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
    Ok(build_dashboard_summary(&data, chrono::Local::now().date_naive()))
}

fn build_dashboard_summary(data: &crate::AppDataDoc, today: chrono::NaiveDate) -> DashboardSummaryDto {
    let rates = crate::rate_map(data);

    let active = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();
    let inactive_count = data.subscriptions.iter().filter(|s| s.inactive).count();

    let total_monthly: Decimal = active
        .iter()
        .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
        .sum();
    let total_yearly = total_monthly * Decimal::from(12);
    let avg_monthly = if active.is_empty() {
        Decimal::ZERO
    } else {
        total_monthly / Decimal::from(active.len())
    };

    // Every projected charge from today to month end (weekly = several), same rule as the Calendar stats.
    let month_end = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .and_then(|first| first.checked_add_months(chrono::Months::new(1)))
        .and_then(|next| next.pred_opt())
        .unwrap_or(today);
    let amount_due_this_month: Decimal = active
        .iter()
        .map(|s| {
            let n = crate::commands::subscriptions::payment_dates_in_window(s, today, month_end).map_or(0, |d| d.len());
            crate::convert_to_main(s.price, &s.currency_id, &rates) * Decimal::from(n)
        })
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
        .max_by(|a, b| a.price.cmp(&b.price));

    let monthly_expenses_total: Decimal = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
        .sum();
    let total_spending = total_monthly + monthly_expenses_total;
    let budget = data.settings.budget;
    let hundred = Decimal::from(100);
    let budget_used = if budget > Decimal::ZERO {
        // Not clamped: >100 drives the over-budget (red) state; the UI clamps the bar width.
        Some(total_spending / budget * hundred)
    } else {
        None
    };
    let budget_left = if budget > Decimal::ZERO {
        Some((budget - total_spending).max(Decimal::ZERO))
    } else {
        None
    };
    let over_budget = if budget > Decimal::ZERO && total_spending > budget {
        Some(total_spending - budget)
    } else {
        None
    };

    let total_savings_monthly: Decimal = data
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
    // Nearest 5 by effective next payment; a past date on an auto-renew sub is projected forward.
    let mut upcoming_subscriptions = active
        .iter()
        .filter_map(|s| {
            let d = parse_date_ymd(&s.next_payment)?;
            let d = if s.auto_renew && d < today {
                crate::commands::subscriptions::first_cycle_date_on_or_after(d, s.cycle as u64, s.frequency as u64, today)?
            } else {
                d
            };
            Some((s, d))
        })
        .filter(|(_, d)| *d >= today && *d <= today + chrono::Duration::days(30))
        .collect::<Vec<_>>();
    upcoming_subscriptions.sort_by_key(|(_, d)| *d);
    let upcoming_subscriptions = upcoming_subscriptions
        .into_iter()
        .take(5)
        .map(|(s, d)| crate::models::SubscriptionDoc {
            next_payment: d.format("%Y-%m-%d").to_string(),
            ..(*s).clone()
        })
        .collect::<Vec<_>>();

    let month_total: Decimal = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, today.year(), today.month()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
        .sum();
    let year_total: Decimal = data
        .expenses
        .iter()
        .filter(|e| expense_in_year(e, today.year()))
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
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

    DashboardSummaryDto {
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
        upcoming_subscriptions,
        expense_aggregation: DashboardExpenseAggregationDto {
            month_total,
            year_total,
            recent_expenses,
        },
    }
}

#[tauri::command]
pub fn get_dashboard_charts(state: State<'_, AppState>) -> Result<DashboardChartsDto, crate::errors::AppError> {
    let data = { let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?; guard.doc()? };
    let rates = crate::rate_map(&data);
    let active = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();

    let category_name = data.categories.iter().map(|c| (c.id.clone(), c.name.clone())).collect::<std::collections::HashMap<_, _>>();
    let payment_name = data.payment_methods.iter().map(|p| (p.id.clone(), p.name.clone())).collect::<std::collections::HashMap<_, _>>();
    let member_name = data.household.iter().map(|h| (h.id.clone(), h.name.clone())).collect::<std::collections::HashMap<_, _>>();

    let mut category_costs: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
    let mut pm_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut member_costs: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
    for s in active {
        let monthly = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
        *category_costs.entry(s.category_id.clone()).or_insert(Decimal::ZERO) += monthly;
        *pm_counts.entry(s.payment_method_id.clone()).or_insert(0) += 1;
        *member_costs.entry(s.payer_user_id.clone()).or_insert(Decimal::ZERO) += monthly;
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
    let next_month: Decimal = active
        .iter()
        .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
        .sum();
    let next_quarter = next_month * Decimal::from(3);
    let now = chrono::Local::now().date_naive();
    let resolved = resolve_month_comparison(&data.expenses, &rates, now);
    let diff = resolved.current_total - resolved.previous_total;
    let hundred = Decimal::from(100);
    let diff_percent = if resolved.previous_total > Decimal::ZERO {
        diff / resolved.previous_total * hundred
    } else if resolved.current_total > Decimal::ZERO {
        hundred
    } else {
        Decimal::ZERO
    };
    Ok(DashboardForecastDto {
        forecast: DashboardForecastInnerDto {
            next_month,
            next_quarter,
            next_month_label: next_month_labels(now, 1).remove(0),
            quarter_labels: next_month_labels(now, 3),
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
    Ok(build_dashboard_trends(&data, chrono::Local::now().date_naive()))
}

fn build_dashboard_trends(data: &crate::AppDataDoc, now: chrono::NaiveDate) -> DashboardTrendsDto {
    let rates = crate::rate_map(data);

    let mut top = data.expenses.clone();
    top.retain(|e| expense_in_calendar_month(e, now.year(), now.month()));
    top.sort_by_key(|b| std::cmp::Reverse(crate::convert_to_main(b.amount, &b.currency_id, &rates)));
    top.truncate(5);

    let month_expenses = data
        .expenses
        .iter()
        .filter(|e| expense_in_calendar_month(e, now.year(), now.month()))
        .collect::<Vec<_>>();
    let total: Decimal = month_expenses
        .iter()
        .map(|e| crate::convert_to_main(e.amount, &e.currency_id, &rates))
        .sum();
    let count = month_expenses.len();
    let avg = if count > 0 {
        total / Decimal::from(count)
    } else {
        Decimal::ZERO
    };

    let mut day_map = [Decimal::ZERO; 7];
    let mut day_count = [0_i64; 7];
    for e in &month_expenses {
        if let Some(d) = e.naive_date() {
            let idx = d.weekday().num_days_from_sunday() as usize;
            day_map[idx] += crate::convert_to_main(e.amount, &e.currency_id, &rates);
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

    let mut tag_map: std::collections::HashMap<String, Decimal> = std::collections::HashMap::new();
    for e in month_expenses {
        let amount = crate::convert_to_main(e.amount, &e.currency_id, &rates);
        for tag in &e.tags {
            *tag_map.entry(tag.clone()).or_insert(Decimal::ZERO) += amount;
        }
    }
    let mut tags = tag_map
        .into_iter()
        .map(|(tag, total)| DashboardTagExpenseStatDto { tag, total })
        .collect::<Vec<_>>();
    tags.sort_by(|a, b| {
        b.total.cmp(&a.total)
    });

    let active_subs = data.subscriptions.iter().filter(|s| !s.inactive).collect::<Vec<_>>();
    let mut spending_history = Vec::new();
    for back in (0..6).rev() {
        let (y, m) = shift_calendar_month(now.year(), now.month(), -back);
        let anchor = chrono::NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let (ny, nm) = shift_calendar_month(y, m, 1);
        let month_end = chrono::NaiveDate::from_ymd_opt(ny, nm, 1).unwrap().pred_opt().unwrap();
        // A subscription only costs money in months it was running: started by month end and
        // not cancelled before the month began (inactive ones need a cancellation date to count).
        let monthly_subs_total: Decimal = data
            .subscriptions
            .iter()
            .filter(|s| {
                let started = parse_date_ymd(&s.start_date).is_none_or(|d| d <= month_end);
                let cancel = s.cancellation_date.as_deref().and_then(parse_date_ymd);
                let running = match cancel {
                    Some(c) => c >= anchor,
                    None => !s.inactive,
                };
                started && running
            })
            .map(|s| crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates)))
            .sum();
        let expenses_total: Decimal = data
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
            total_paid: monthly * Decimal::from(months_active),
            monthly_equivalent: monthly,
        }
    }).collect::<Vec<_>>();

    let mut cat_map: std::collections::HashMap<String, (Decimal, i64)> = std::collections::HashMap::new();
    for s in &active_subs {
        let monthly = crate::price_per_month(s.cycle, s.frequency, crate::convert_to_main(s.price, &s.currency_id, &rates));
        let item = cat_map.entry(s.category_id.clone()).or_insert((Decimal::ZERO, 0));
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
                total_monthly / Decimal::from(count)
            } else {
                Decimal::ZERO
            },
        })
        .collect::<Vec<_>>();

    DashboardTrendsDto {
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
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::*;
    use crate::models::ExpenseDoc;

    fn expense_row(y: i32, m: u32, d: u32, amount: Decimal, currency_id: &str) -> ExpenseDoc {
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
        let rates = std::collections::HashMap::from([("USD".into(), Decimal::ONE)]);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        let expenses = vec![
            expense_row(2026, 5, 3, Decimal::from(10), "USD"),
            expense_row(2026, 4, 3, Decimal::from(20), "USD"),
        ];
        let r = resolve_month_comparison(&expenses, &rates, today);
        assert_eq!(r.style, MonthComparisonStyle::Mtd, "MTD should win");
        assert_eq!(r.current_total, Decimal::from(10));
        assert_eq!(r.previous_total, Decimal::from(20));
        assert_eq!(r.current_count, 1);
        assert_eq!(r.previous_count, 1);
    }

    #[test]
    fn resolve_comparison_uses_completed_pair_when_mtd_empty_early_in_month() {
        let rates = std::collections::HashMap::from([("USD".into(), Decimal::ONE)]);
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        let expenses = vec![
            expense_row(2026, 5, 15, Decimal::from(100), "USD"),
            expense_row(2026, 4, 20, Decimal::from(200), "USD"),
        ];
        let r = resolve_month_comparison(&expenses, &rates, today);
        assert_eq!(r.style, MonthComparisonStyle::CompletedPair);
        assert_eq!(r.current_total, Decimal::from(200), "April M-1");
        assert_eq!(r.previous_total, Decimal::ZERO, "March M-2");
        assert_eq!(r.current_month_label, "2026-04");
        assert_eq!(r.previous_month_label, "2026-03");
        assert_eq!(r.current_count, 1);
        assert_eq!(r.previous_count, 0);
    }

    #[test]
    fn next_month_labels_roll_over_year() {
        let nov = chrono::NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();
        assert_eq!(next_month_labels(nov, 1), ["2026-12"]);
        assert_eq!(next_month_labels(nov, 3), ["2026-12", "2027-01", "2027-02"]);
    }

    fn doc_with_rate_two() -> crate::AppDataDoc {
        let mut doc = crate::test_support::base_seeded_doc().expect("doc");
        doc.currencies[1].rate = Decimal::from(2);
        doc
    }

    #[test]
    fn summary_upcoming_is_nearest_first_and_projects_auto_renew() {
        let mut doc = doc_with_rate_two();
        let base = crate::test_support::doc_with_restart_sensitive_fields()
            .expect("doc")
            .subscriptions
            .last()
            .cloned()
            .expect("sub");
        let sub = |id: &str, next: &str, auto_renew: bool, inactive: bool| crate::models::SubscriptionDoc {
            id: id.into(),
            next_payment: next.into(),
            auto_renew,
            inactive,
            ..base.clone()
        };
        doc.subscriptions = vec![
            sub("a", "2026-05-25", false, false),
            sub("b", "2026-05-12", false, false),
            sub("c", "2026-05-15", true, false),
            sub("d", "2026-04-11", true, false), // monthly auto-renew -> 2026-05-11
            sub("e", "2026-05-01", false, false), // past, manual -> not upcoming
            sub("f", "2026-05-11", false, true),  // inactive
            sub("g", "2026-07-01", false, false), // beyond 30 days
        ];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let s = build_dashboard_summary(&doc, today);
        let ids = s.upcoming_subscriptions.iter().map(|x| x.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, ["d", "b", "c", "a"]);
        assert_eq!(s.upcoming_subscriptions[0].next_payment, "2026-05-11");
    }

    #[test]
    fn summary_and_trends_convert_mixed_currencies_to_main() {
        let mut doc = doc_with_rate_two();
        let usd = doc.currencies[1].id.clone();
        let eur = doc.currencies[0].id.clone();
        let mut big_usd = expense_row(2026, 5, 4, Decimal::from(30), &usd); // 15 in main
        big_usd.tags = vec!["t".into()];
        let mut eur_row = expense_row(2026, 5, 5, Decimal::from(20), &eur); // 20 in main
        eur_row.tags = vec!["t".into()];
        let old = expense_row(2026, 2, 3, Decimal::from(4), &usd); // 2 in main, same year
        doc.expenses = vec![big_usd, eur_row, old];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();

        let s = build_dashboard_summary(&doc, today);
        assert_eq!(s.expense_aggregation.month_total, Decimal::from(35));
        assert_eq!(s.expense_aggregation.year_total, Decimal::from(37));

        let t = build_dashboard_trends(&doc, today);
        assert_eq!(t.top_expenses[0].currency_id, eur, "20 EUR outranks 30 USD (=15)");
        let dow_total: Decimal = t.day_of_week_stats.iter().map(|d| d.total).sum();
        assert_eq!(dow_total, Decimal::from(35));
        assert_eq!(t.tag_expense_stats[0].total, Decimal::from(35));
    }

    #[test]
    fn amount_due_counts_every_charge_left_this_month() {
        let mut doc = doc_with_rate_two();
        let base = crate::test_support::doc_with_restart_sensitive_fields().expect("doc").subscriptions.last().cloned().expect("sub");
        doc.subscriptions = vec![crate::models::SubscriptionDoc {
            id: "weekly".into(),
            price: Decimal::from(10),
            currency_id: doc.currencies[0].id.clone(),
            cycle: 2,
            frequency: 1,
            next_payment: "2026-05-11".into(),
            inactive: false,
            ..base
        }];
        let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let s = build_dashboard_summary(&doc, today);
        // 11, 18, 25 May; main currency (rate 1).
        assert_eq!(s.amount_due_this_month, Decimal::from(30));
    }

    #[test]
    fn spending_history_skips_months_before_start_and_after_cancel() {
        let mut doc = doc_with_rate_two();
        let base = crate::test_support::doc_with_restart_sensitive_fields().expect("doc").subscriptions.last().cloned().expect("sub");
        let monthly = |id: &str, start: &str, cancel: Option<&str>, inactive: bool| crate::models::SubscriptionDoc {
            id: id.into(),
            price: Decimal::from(10),
            currency_id: doc.currencies[0].id.clone(),
            cycle: 3,
            frequency: 1,
            start_date: start.into(),
            cancellation_date: cancel.map(Into::into),
            inactive,
            ..base.clone()
        };
        doc.subscriptions = vec![
            monthly("new", "2026-04-20", None, false),
            monthly("gone", "2025-01-01", Some("2026-02-15"), true),
        ];
        doc.expenses.clear();
        let t = build_dashboard_trends(&doc, chrono::NaiveDate::from_ymd_opt(2026, 5, 10).unwrap());
        let per_month = |m0: u32| t.spending_history.iter().find(|h| h.month == m0).unwrap().amount;
        let per_month_new = Decimal::from(10); // monthly cycle, main currency
        assert_eq!(per_month(0), per_month_new, "Jan: only the cancelled one ran");
        assert_eq!(per_month(1), per_month_new, "Feb: cancelled mid-month still counts");
        assert_eq!(per_month(2), Decimal::ZERO, "Mar: nothing running");
        assert_eq!(per_month(3), per_month_new, "Apr: new one started");
    }
}
