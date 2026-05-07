use chrono::Datelike;
use serde::{Deserialize, Serialize};

/// Subset of preferences persisted inside the Redb app snapshot (`AppDataDoc.settings`).
/// Full UI settings (themes, view modes, notifications text, etc.) live in the separate config store
/// (`config_get` / `config_set`, key `settings` on the frontend) and are **not** mirrored here.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsDoc {
    #[serde(default)]
    pub(crate) budget: f64,
    #[serde(default)]
    pub(crate) main_currency_id: String,
    #[serde(default)]
    pub(crate) currency_update_targets: Vec<String>,
    #[serde(default = "default_rate_days")]
    pub(crate) rate_history_days: u32,
}

fn default_rate_days() -> u32 {
    90
}

fn default_rates_provider() -> String {
    "frankfurter".to_string()
}

fn default_legacy_updated_at() -> i64 {
    946_684_800_000 // 2000-01-01T00:00:00Z
}

/// Login / password / TOTP secret — not stored in `SubscriptionDoc` or sync snapshots; persisted in the OS keyring
/// (`secure_storage.subscription_credentials:{id}`) when saving via [`SubscriptionInputDto`].
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionCredentialsDto {
    #[serde(default)]
    pub(crate) login: String,
    #[serde(default)]
    pub(crate) password: String,
    #[serde(default)]
    pub(crate) totp_secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionDoc {
    pub(crate) id: String,
    #[serde(default = "default_legacy_updated_at")]
    pub(crate) updated_at: i64,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) logo: String,
    #[serde(default)]
    pub(crate) price: f64,
    #[serde(default)]
    pub(crate) currency_id: String,
    #[serde(default)]
    pub(crate) next_payment: String,
    #[serde(default)]
    pub(crate) start_date: String,
    #[serde(default = "default_cycle")]
    pub(crate) cycle: u8,
    #[serde(default = "default_frequency")]
    pub(crate) frequency: u32,
    #[serde(default)]
    pub(crate) inactive: bool,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) payer_user_id: String,
    #[serde(default)]
    pub(crate) cancellation_date: Option<String>,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) notify: bool,
    #[serde(default)]
    pub(crate) notify_days_before: i64,
    #[serde(default)]
    pub(crate) last_notified_date: String,
    #[serde(default)]
    pub(crate) auto_renew: bool,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) replacement_subscription_id: Option<String>,
    #[serde(default)]
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) favorite: bool,
    #[serde(default)]
    pub(crate) payment_history: Vec<PaymentRecordDto>,
}

fn default_cycle() -> u8 {
    3
}

fn default_frequency() -> u32 {
    1
}

fn default_auto_renew() -> bool {
    true
}

fn default_notify_true() -> bool {
    true
}

fn default_notify_days_before() -> i64 {
    1
}

fn is_supported_date_input(raw: &str) -> bool {
    parse_loose_date_to_ymd(raw).is_ok()
}

/// Normalizes expense `created_at` to RFC3339 (accepts RFC3339 or `YYYY-MM-DD`).
pub(crate) fn normalize_expense_timestamp(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("field_invalid_date:createdAt".to_string());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.to_rfc3339());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let ndt = d
            .and_hms_opt(12, 0, 0)
            .ok_or_else(|| "field_invalid_date:createdAt".to_string())?;
        return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc).to_rfc3339());
    }
    if trimmed.len() >= 10 {
        let head = &trimmed[..10];
        if let Ok(d) = chrono::NaiveDate::parse_from_str(head, "%Y-%m-%d") {
            let ndt = d
                .and_hms_opt(12, 0, 0)
                .ok_or_else(|| "field_invalid_date:createdAt".to_string())?;
            return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc).to_rfc3339());
        }
    }
    Err("field_invalid_date:createdAt".to_string())
}

pub(crate) fn ymd_to_utc_noon_rfc3339(year: i32, month: u32, day: u32) -> Result<String, String> {
    let d = chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "field_invalid_date:createdAt".to_string())?;
    let ndt = d
        .and_hms_opt(12, 0, 0)
        .ok_or_else(|| "field_invalid_date:createdAt".to_string())?;
    Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc).to_rfc3339())
}

/// Parse legacy date strings (ISO / RFC3339 / first 10 chars) into calendar components.
pub(crate) fn parse_loose_date_to_ymd(raw: &str) -> Result<(i32, u32, u32), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("field_invalid_date:date".to_string());
    }
    let d = if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        date
    } else if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        date_time.date_naive()
    } else if trimmed.len() >= 10 {
        chrono::NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").map_err(|_| "field_invalid_date:date".to_string())?
    } else {
        return Err("field_invalid_date:date".to_string());
    };
    Ok((d.year(), d.month(), d.day()))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpenseDoc {
    pub(crate) id: String,
    #[serde(default = "default_legacy_updated_at")]
    pub(crate) updated_at: i64,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) amount: f64,
    #[serde(default)]
    pub(crate) currency_id: String,
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) payer_user_id: String,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) subscription_id: String,
    #[serde(default)]
    pub(crate) payment_record_id: String,
}

impl ExpenseDoc {
    pub(crate) fn naive_date(&self) -> Option<chrono::NaiveDate> {
        parse_loose_date_to_ymd(&self.created_at)
            .ok()
            .and_then(|(y, m, d)| chrono::NaiveDate::from_ymd_opt(y, m, d))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrencyDoc {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) symbol: String,
    #[serde(default)]
    pub(crate) code: String,
    #[serde(default = "default_rate")]
    pub(crate) rate: f64,
    #[serde(default)]
    pub(crate) sort_order: i64,
    #[serde(default)]
    pub(crate) i18n_key: String,
}

fn default_rate() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryDoc {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) icon: String,
    #[serde(default)]
    pub(crate) sort_order: i64,
    #[serde(default)]
    pub(crate) i18n_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaymentMethodDoc {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) icon: String,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) sort_order: i64,
    #[serde(default)]
    pub(crate) i18n_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HouseholdMemberDoc {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) email: String,
    #[serde(default)]
    pub(crate) sort_order: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagDoc {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) favorite: bool,
    #[serde(default)]
    pub(crate) sort_order: i64,
    #[serde(default)]
    pub(crate) i18n_key: String,
}

fn default_true() -> bool {
    true
}

/// Record of a user-initiated delete for sync (LWW against remote rows by `updated_at` or unconditional for catalog).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeletionTombstone {
    pub(crate) entity_kind: TombstoneEntityKind,
    pub(crate) entity_id: String,
    pub(crate) deleted_at: i64,
    #[serde(default)]
    pub(crate) device_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TombstoneEntityKind {
    Subscription,
    Expense,
    Category,
    Currency,
    Household,
    PaymentMethod,
    Tag,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppDataDoc {
    #[serde(default)]
    pub(crate) subscriptions: Vec<SubscriptionDoc>,
    #[serde(default)]
    pub(crate) expenses: Vec<ExpenseDoc>,
    #[serde(default)]
    pub(crate) categories: Vec<CategoryDoc>,
    #[serde(default)]
    pub(crate) currencies: Vec<CurrencyDoc>,
    #[serde(default)]
    pub(crate) household: Vec<HouseholdMemberDoc>,
    #[serde(default)]
    pub(crate) payment_methods: Vec<PaymentMethodDoc>,
    #[serde(default)]
    pub(crate) tags: Vec<TagDoc>,
    pub(crate) settings: SettingsDoc,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppConfigDoc {
    #[serde(default)]
    pub(crate) rates_api_key: String,
    #[serde(default = "default_rates_provider")]
    pub(crate) rates_provider: String,
    #[serde(default)]
    pub(crate) fixer_api_key: String,
    #[serde(default)]
    pub(crate) fixer_provider: i64,
    #[serde(default)]
    pub(crate) telegram_bot_token: String,
    #[serde(default)]
    pub(crate) telegram_chat_id: String,
    #[serde(default)]
    pub(crate) telegram_proxy_url: String,
    #[serde(default)]
    pub(crate) telegram_enabled: bool,
    #[serde(default)]
    pub(crate) initialized: bool,
}

impl Default for AppConfigDoc {
    fn default() -> Self {
        Self {
            rates_api_key: String::new(),
            rates_provider: default_rates_provider(),
            fixer_api_key: String::new(),
            fixer_provider: 0,
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            telegram_proxy_url: String::new(),
            telegram_enabled: false,
            initialized: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RatePoint {
    pub(crate) rate: f64,
    pub(crate) recorded_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogsLoadDto {
    pub(crate) categories: Vec<CategoryDoc>,
    pub(crate) currencies: Vec<CurrencyDoc>,
    pub(crate) household: Vec<HouseholdMemberDoc>,
    pub(crate) payment_methods: Vec<PaymentMethodDoc>,
    pub(crate) tags: Vec<TagDoc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogsUsageSummaryDto {
    pub(crate) category_usage: std::collections::HashMap<String, u64>,
    pub(crate) currency_usage: std::collections::HashMap<String, u64>,
    pub(crate) payment_method_usage: std::collections::HashMap<String, u64>,
    pub(crate) tag_usage: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionListItemDto {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) logo: String,
    #[serde(default)]
    pub(crate) price: f64,
    #[serde(default)]
    pub(crate) currency_id: String,
    #[serde(default)]
    pub(crate) next_payment: String,
    #[serde(default)]
    pub(crate) start_date: String,
    #[serde(default = "default_cycle")]
    pub(crate) cycle: u8,
    #[serde(default = "default_frequency")]
    pub(crate) frequency: u32,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) payer_user_id: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) notify: bool,
    #[serde(default)]
    pub(crate) notify_days_before: i64,
    #[serde(default)]
    pub(crate) last_notified_date: String,
    #[serde(default)]
    pub(crate) inactive: bool,
    #[serde(default)]
    pub(crate) auto_renew: bool,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) cancellation_date: Option<String>,
    #[serde(default)]
    pub(crate) replacement_subscription_id: Option<String>,
    #[serde(default)]
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) favorite: bool,
    #[serde(default)]
    pub(crate) payment_history: Vec<PaymentRecordDto>,
    #[serde(default)]
    pub(crate) monthly_price: f64,
    #[serde(default)]
    pub(crate) days_left: u64,
    #[serde(default)]
    pub(crate) overdue: bool,
    #[serde(default)]
    pub(crate) credentials: Option<SubscriptionCredentialsDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionInputDto {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) price: f64,
    pub(crate) currency_id: String,
    pub(crate) next_payment: String,
    pub(crate) start_date: String,
    #[serde(default)]
    pub(crate) logo: String,
    #[serde(default = "default_cycle")]
    pub(crate) cycle: u8,
    #[serde(default = "default_frequency")]
    pub(crate) frequency: u32,
    #[serde(default)]
    pub(crate) inactive: bool,
    #[serde(default = "default_auto_renew")]
    pub(crate) auto_renew: bool,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) payer_user_id: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default = "default_notify_true")]
    pub(crate) notify: bool,
    #[serde(default = "default_notify_days_before")]
    pub(crate) notify_days_before: i64,
    #[serde(default)]
    pub(crate) last_notified_date: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) cancellation_date: Option<String>,
    #[serde(default)]
    pub(crate) replacement_subscription_id: Option<String>,
    #[serde(default)]
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) favorite: bool,
    #[serde(default)]
    pub(crate) payment_history: Vec<PaymentRecordDto>,
    #[serde(default)]
    pub(crate) credentials: Option<SubscriptionCredentialsDto>,
}

impl SubscriptionInputDto {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("field_required:name".to_string());
        }
        if !self.price.is_finite() || self.price < 0.0 {
            return Err("field_invalid_number:price".to_string());
        }
        if self.currency_id.trim().is_empty() {
            return Err("field_required:currencyId".to_string());
        }
        if !is_supported_date_input(&self.next_payment) {
            return Err("field_invalid_date:nextPayment".to_string());
        }
        if !is_supported_date_input(&self.start_date) {
            return Err("field_invalid_date:startDate".to_string());
        }
        Ok(())
    }

}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpenseInputDto {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) amount: f64,
    pub(crate) currency_id: String,
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) payer_user_id: String,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) subscription_id: String,
    #[serde(default)]
    pub(crate) payment_record_id: String,
}

impl ExpenseInputDto {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("field_required:name".to_string());
        }
        if !self.amount.is_finite() || self.amount < 0.0 {
            return Err("field_invalid_number:amount".to_string());
        }
        if self.currency_id.trim().is_empty() {
            return Err("field_required:currencyId".to_string());
        }
        if self.created_at.trim().is_empty() {
            return Err("field_required:createdAt".to_string());
        }
        normalize_expense_timestamp(&self.created_at)?;
        Ok(())
    }

}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CurrencyRateUpdateDto {
    pub(crate) id: String,
    pub(crate) rate: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaymentRecordDto {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) date: String,
    #[serde(default)]
    pub(crate) amount: f64,
    #[serde(default)]
    pub(crate) currency_id: String,
    #[serde(default)]
    pub(crate) note: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubscriptionsPageRequestDto {
    #[serde(default)]
    pub(crate) search: String,
    #[serde(default)]
    pub(crate) state: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) tag: String,
    #[serde(default)]
    pub(crate) sort_by: String,
    #[serde(default)]
    pub(crate) disabled_to_bottom: bool,
    #[serde(default)]
    pub(crate) hide_disabled: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpensesPageRequestDto {
    #[serde(default)]
    pub(crate) search: String,
    #[serde(default)]
    pub(crate) category_id: String,
    #[serde(default)]
    pub(crate) payment_method_id: String,
    #[serde(default)]
    pub(crate) tag: String,
    #[serde(default)]
    pub(crate) date_from: String,
    #[serde(default)]
    pub(crate) date_to: String,
    #[serde(default)]
    pub(crate) sort_by: String,
    #[serde(default)]
    pub(crate) limit: usize,
    #[serde(default)]
    pub(crate) offset: usize,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExpensesPageResponseDto {
    pub(crate) items: Vec<ExpenseDoc>,
    pub(crate) total: usize,
}
