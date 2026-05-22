//! One-time postcard migration from legacy `f64` money fields to `Decimal` (string encoding).

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;

use crate::errors::{AppError, AppResult};
use crate::money::{from_f64_legacy, MIGRATION_KV_KEY};
use crate::models::{
    CurrencyDoc, ExpenseDoc, PaymentRecordDto, RatePoint, SettingsDoc, SubscriptionDoc,
};
use crate::state::{decode_bin, kv_set_in_tx, run_write_transaction, T2_CURRENCIES, T2_EXPENSES, T2_RATE_HISTORY, T2_SETTINGS, T2_SUBSCRIPTIONS};

pub(crate) mod legacy {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PaymentRecordDtoV1 {
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

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SubscriptionDocV1 {
        pub(crate) id: String,
        #[serde(default)]
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
        #[serde(default)]
        pub(crate) cycle: u8,
        #[serde(default)]
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
        pub(crate) payment_history: Vec<PaymentRecordDtoV1>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ExpenseDocV1 {
        pub(crate) id: String,
        #[serde(default)]
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

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CurrencyDocV1 {
        pub(crate) id: String,
        #[serde(default)]
        pub(crate) name: String,
        #[serde(default)]
        pub(crate) symbol: String,
        #[serde(default)]
        pub(crate) code: String,
        #[serde(default = "super::default_rate_f64")]
        pub(crate) rate: f64,
        #[serde(default)]
        pub(crate) sort_order: i64,
        #[serde(default)]
        pub(crate) i18n_key: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SettingsDocV1 {
        #[serde(default)]
        pub(crate) budget: f64,
        #[serde(default)]
        pub(crate) main_currency_id: String,
        #[serde(default)]
        pub(crate) currency_update_targets: Vec<String>,
        #[serde(default = "super::default_rate_days")]
        pub(crate) rate_history_days: u32,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct RatePointV1 {
        pub(crate) rate: f64,
        pub(crate) recorded_at: String,
    }
}

fn default_rate_f64() -> f64 {
    1.0
}

fn default_rate_days() -> u32 {
    90
}

fn decode_row<T, L, F>(raw: &[u8], convert: F) -> AppResult<T>
where
    T: DeserializeOwned,
    L: DeserializeOwned,
    F: FnOnce(L) -> T,
{
    match decode_bin::<T>(raw) {
        Ok(v) => Ok(v),
        Err(_) => {
            let legacy = decode_bin::<L>(raw)?;
            Ok(convert(legacy))
        }
    }
}

fn read_money_table<T, L, F>(
    db: &Database,
    table: TableDefinition<&str, &[u8]>,
    convert: F,
) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
    L: DeserializeOwned,
    F: Fn(L) -> T + Copy,
{
    let tx = db.begin_read().map_err(AppError::from)?;
    let t = match tx.open_table(table) {
        Ok(t) => t,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    for entry in t.iter().map_err(AppError::from)? {
        let (_, value) = entry.map_err(AppError::from)?;
        rows.push(decode_row::<T, L, _>(value.value(), convert)?);
    }
    Ok(rows)
}

impl From<legacy::PaymentRecordDtoV1> for PaymentRecordDto {
    fn from(v: legacy::PaymentRecordDtoV1) -> Self {
        Self {
            id: v.id,
            date: v.date,
            amount: from_f64_legacy(v.amount),
            currency_id: v.currency_id,
            note: v.note,
        }
    }
}

impl From<legacy::SubscriptionDocV1> for SubscriptionDoc {
    fn from(v: legacy::SubscriptionDocV1) -> Self {
        Self {
            id: v.id,
            updated_at: v.updated_at,
            name: v.name,
            logo: v.logo,
            price: from_f64_legacy(v.price),
            currency_id: v.currency_id,
            next_payment: v.next_payment,
            start_date: v.start_date,
            cycle: v.cycle,
            frequency: v.frequency,
            inactive: v.inactive,
            category_id: v.category_id,
            payment_method_id: v.payment_method_id,
            payer_user_id: v.payer_user_id,
            cancellation_date: v.cancellation_date,
            notes: v.notes,
            notify: v.notify,
            notify_days_before: v.notify_days_before,
            last_notified_date: v.last_notified_date,
            auto_renew: v.auto_renew,
            url: v.url,
            replacement_subscription_id: v.replacement_subscription_id,
            created_at: v.created_at,
            tags: v.tags,
            favorite: v.favorite,
            payment_history: v.payment_history.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<legacy::ExpenseDocV1> for ExpenseDoc {
    fn from(v: legacy::ExpenseDocV1) -> Self {
        Self {
            id: v.id,
            updated_at: v.updated_at,
            name: v.name,
            amount: from_f64_legacy(v.amount),
            currency_id: v.currency_id,
            created_at: v.created_at,
            category_id: v.category_id,
            tags: v.tags,
            payment_method_id: v.payment_method_id,
            payer_user_id: v.payer_user_id,
            notes: v.notes,
            url: v.url,
            subscription_id: v.subscription_id,
            payment_record_id: v.payment_record_id,
        }
    }
}

impl From<legacy::CurrencyDocV1> for CurrencyDoc {
    fn from(v: legacy::CurrencyDocV1) -> Self {
        Self {
            id: v.id,
            name: v.name,
            symbol: v.symbol,
            code: v.code,
            rate: from_f64_legacy(v.rate),
            sort_order: v.sort_order,
            i18n_key: v.i18n_key,
        }
    }
}

impl From<legacy::SettingsDocV1> for SettingsDoc {
    fn from(v: legacy::SettingsDocV1) -> Self {
        Self {
            budget: from_f64_legacy(v.budget),
            main_currency_id: v.main_currency_id,
            currency_update_targets: v.currency_update_targets,
            rate_history_days: v.rate_history_days,
        }
    }
}

impl From<legacy::RatePointV1> for RatePoint {
    fn from(v: legacy::RatePointV1) -> Self {
        Self {
            rate: from_f64_legacy(v.rate),
            recorded_at: v.recorded_at,
        }
    }
}

pub(crate) fn load_subscriptions_migrated(db: &Database) -> AppResult<Vec<SubscriptionDoc>> {
    read_money_table::<SubscriptionDoc, legacy::SubscriptionDocV1, _>(db, T2_SUBSCRIPTIONS, Into::into)
}

pub(crate) fn load_expenses_migrated(db: &Database) -> AppResult<Vec<ExpenseDoc>> {
    read_money_table::<ExpenseDoc, legacy::ExpenseDocV1, _>(db, T2_EXPENSES, Into::into)
}

pub(crate) fn load_currencies_migrated(db: &Database) -> AppResult<Vec<CurrencyDoc>> {
    read_money_table::<CurrencyDoc, legacy::CurrencyDocV1, _>(db, T2_CURRENCIES, Into::into)
}

pub(crate) fn load_settings_migrated(db: &Database) -> AppResult<SettingsDoc> {
    let tx = db.begin_read().map_err(AppError::from)?;
    let t = tx.open_table(T2_SETTINGS).map_err(AppError::from)?;
    let entry = t.get(crate::state::KEY_DATA).map_err(AppError::from)?;
    match entry {
        None => Ok(crate::state::default_settings_doc()),
        Some(v) => decode_row::<SettingsDoc, legacy::SettingsDocV1, _>(v.value(), Into::into),
    }
}

pub(crate) fn load_rate_history_migrated(
    db: &Database,
) -> AppResult<std::collections::HashMap<String, Vec<RatePoint>>> {
    let tx = db.begin_read().map_err(AppError::from)?;
    let t = match tx.open_table(T2_RATE_HISTORY) {
        Ok(t) => t,
        Err(_) => return Ok(std::collections::HashMap::new()),
    };
    let entry = t.get(crate::state::KEY_DATA).map_err(AppError::from)?;
    match entry {
        None => Ok(std::collections::HashMap::new()),
        Some(v) => {
            match decode_bin::<std::collections::HashMap<String, Vec<RatePoint>>>(v.value()) {
                Ok(m) => Ok(m),
                Err(_) => {
                    let legacy: std::collections::HashMap<String, Vec<legacy::RatePointV1>> =
                        decode_bin(v.value())?;
                    Ok(legacy
                        .into_iter()
                        .map(|(k, pts)| (k, pts.into_iter().map(Into::into).collect()))
                        .collect())
                }
            }
        }
    }
}

pub(crate) fn migration_marker_done(db: &Database) -> AppResult<bool> {
    let tx = db.begin_read().map_err(AppError::from)?;
    let table = match tx.open_table(crate::KV_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };
    Ok(table
        .get(MIGRATION_KV_KEY)
        .map_err(AppError::from)?
        .map(|v| v.value() == "done")
        .unwrap_or(false))
}

/// Re-persist all money-bearing tables once after upgrade; sets KV marker.
pub(crate) fn finalize_money_migration_if_needed(
    db: &Database,
    data: &crate::AppDataDoc,
    cfg: &crate::models::AppConfigDoc,
) -> AppResult<bool> {
    if migration_marker_done(db)? {
        return Ok(false);
    }
    crate::state::save_app_data_typed(db, data, cfg)?;
    let history = load_rate_history_migrated(db)?;
    if !history.is_empty() {
        crate::state::write_singleton_bin_typed(db, T2_RATE_HISTORY, &history)?;
    }
    let tx = db.begin_write().map_err(AppError::from)?;
    let _ = tx.open_table(crate::KV_TABLE).map_err(AppError::from)?;
    tx.commit().map_err(AppError::from)?;
    run_write_transaction(db, |tx| kv_set_in_tx(tx, MIGRATION_KV_KEY, "done"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::legacy;
    use crate::models::{ExpenseDoc, SubscriptionDoc};
    use crate::state::{decode_bin, encode_bin};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn legacy_subscription_converts_price() {
        let v1 = legacy::SubscriptionDocV1 {
            id: "s1".into(),
            updated_at: 1,
            name: "Test".into(),
            logo: String::new(),
            price: 9.99,
            currency_id: "cur-1".into(),
            next_payment: "2026-06-01".into(),
            start_date: "2026-01-01".into(),
            cycle: 3,
            frequency: 1,
            inactive: false,
            category_id: "cat-1".into(),
            payment_method_id: String::new(),
            payer_user_id: String::new(),
            cancellation_date: None,
            notes: String::new(),
            notify: true,
            notify_days_before: 1,
            last_notified_date: String::new(),
            auto_renew: true,
            url: String::new(),
            replacement_subscription_id: None,
            created_at: String::new(),
            tags: vec![],
            favorite: false,
            payment_history: vec![],
        };
        let bytes = encode_bin(&v1).unwrap();
        let legacy: legacy::SubscriptionDocV1 = decode_bin(&bytes).unwrap();
        let doc: SubscriptionDoc = legacy.into();
        assert_eq!(doc.price, crate::money::from_f64_legacy(9.99));
    }

    #[test]
    fn decimal_sum_avoids_float_drift() {
        let a = Decimal::from_str("0.1").unwrap();
        let b = Decimal::from_str("0.2").unwrap();
        assert_eq!(a + b, Decimal::from_str("0.3").unwrap());
    }

    #[test]
    fn legacy_expense_converts_amount() {
        let v1 = legacy::ExpenseDocV1 {
            id: "e1".into(),
            updated_at: 1,
            name: "Coffee".into(),
            amount: 12.5,
            currency_id: "cur-1".into(),
            created_at: "2026-05-01T12:00:00.000Z".into(),
            category_id: String::new(),
            tags: vec![],
            payment_method_id: String::new(),
            payer_user_id: String::new(),
            notes: String::new(),
            url: String::new(),
            subscription_id: String::new(),
            payment_record_id: String::new(),
        };
        let bytes = encode_bin(&v1).unwrap();
        let legacy: legacy::ExpenseDocV1 = decode_bin(&bytes).unwrap();
        let doc: ExpenseDoc = legacy.into();
        assert_eq!(doc.amount, Decimal::from_str("12.5").unwrap());
    }
}
