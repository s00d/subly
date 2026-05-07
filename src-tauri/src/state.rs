use std::sync::{Arc, Mutex};

use redb::{
    Database, ReadableDatabase, ReadableMultimapTable, ReadableTable, TableDefinition, WriteTransaction,
};
use redb::MultimapTableDefinition;
use serde::{de::DeserializeOwned, Serialize};
use crate::models::{
    AppConfigDoc, AppDataDoc, CategoryDoc, CurrencyDoc, DeletionTombstone, ExpenseDoc, HouseholdMemberDoc,
    PaymentMethodDoc, SettingsDoc, SubscriptionDoc, TagDoc, TombstoneEntityKind,
};

pub struct AppStateInner {
    pub db: Arc<Database>,
    pub app_data: crate::AppDataDoc,
}

pub type AppState = Mutex<AppStateInner>;

const KEY_DATA: &str = "data";

pub(crate) const T2_SUBSCRIPTIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("subscriptions_v2");
pub(crate) const T2_EXPENSES: TableDefinition<&str, &[u8]> = TableDefinition::new("expenses_v2");
pub(crate) const T2_CATEGORIES: TableDefinition<&str, &[u8]> = TableDefinition::new("categories_v2");
pub(crate) const T2_CURRENCIES: TableDefinition<&str, &[u8]> = TableDefinition::new("currencies_v2");
pub(crate) const T2_HOUSEHOLD: TableDefinition<&str, &[u8]> = TableDefinition::new("household_v2");
pub(crate) const T2_PAYMENT_METHODS: TableDefinition<&str, &[u8]> = TableDefinition::new("payment_methods_v2");
pub(crate) const T2_TAGS: TableDefinition<&str, &[u8]> = TableDefinition::new("tags_v2");
pub(crate) const T2_SETTINGS: TableDefinition<&str, &[u8]> = TableDefinition::new("settings_v2");
pub(crate) const T2_CONFIG: TableDefinition<&str, &[u8]> = TableDefinition::new("config_v2");
pub(crate) const T2_RATE_HISTORY: TableDefinition<&str, &[u8]> = TableDefinition::new("rate_history_v2");
pub(crate) const T2_DELETION_TOMBSTONES: TableDefinition<&str, &[u8]> = TableDefinition::new("deletion_tombstones_v2");
/// Day key `YYYY-MM-DD` (lexicographic) → expense id; accelerates date-bounded listing.
pub(crate) const T2_EXPENSE_IDS_BY_DAY: MultimapTableDefinition<&str, &str> =
    MultimapTableDefinition::new("expense_ids_by_day_v1");

pub use crate::state_tables::EntityTable;

pub(crate) fn expense_day_key_iso(e: &ExpenseDoc) -> Option<String> {
    e.naive_date().map(|d| d.format("%Y-%m-%d").to_string())
}

/// Replace `T2_EXPENSE_IDS_BY_DAY` from an in-memory expense list (same committed txn as table writes).
pub(crate) fn rewrite_expense_day_index_in_tx(tx: &WriteTransaction, expenses: &[ExpenseDoc]) -> Result<(), String> {
    let mut to_remove: Vec<(String, String)> = Vec::new();
    if let Ok(table) = tx.open_multimap_table(T2_EXPENSE_IDS_BY_DAY) {
        for ent in table.iter().map_err(|e| e.to_string())? {
            let (k, mut vals) = ent.map_err(|e| e.to_string())?;
            let k = k.value().to_string();
            while let Some(v) = vals.next() {
                let g = v.map_err(|e| e.to_string())?;
                to_remove.push((k.clone(), g.value().to_string()));
            }
        }
    }
    let mut mm = tx.open_multimap_table(T2_EXPENSE_IDS_BY_DAY).map_err(|e| e.to_string())?;
    for (k, id) in to_remove {
        let _ = mm.remove(k.as_str(), id.as_str()).map_err(|e| e.to_string())?;
    }
    for e in expenses {
        if let Some(day) = expense_day_key_iso(e) {
            mm.insert(day.as_str(), e.id.as_str()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub(crate) fn update_expense_day_index(db: &Database, old: Option<&ExpenseDoc>, new: Option<&ExpenseDoc>) -> Result<(), String> {
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut mm = tx.open_multimap_table(T2_EXPENSE_IDS_BY_DAY).map_err(|e| e.to_string())?;
        if let Some(o) = old {
            if let Some(day) = expense_day_key_iso(o) {
                let _ = mm.remove(day.as_str(), o.id.as_str()).map_err(|e| e.to_string())?;
            }
        }
        if let Some(n) = new {
            if let Some(day) = expense_day_key_iso(n) {
                mm.insert(day.as_str(), n.id.as_str()).map_err(|e| e.to_string())?;
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

/// Expense ids whose calendar day is in `[from, to]` (inclusive), using the day index.
pub(crate) fn expense_ids_in_day_range(
    db: &Database,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Result<std::collections::HashSet<String>, String> {
    use std::collections::HashSet;
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = match tx.open_multimap_table(T2_EXPENSE_IDS_BY_DAY) {
        Ok(t) => t,
        Err(_) => return Ok(HashSet::new()),
    };
    let mut out = HashSet::new();
    let mut d = from;
    while d <= to {
        let key = d.format("%Y-%m-%d").to_string();
        if let Ok(values) = table.get(key.as_str()) {
            for v in values {
                out.insert(v.map_err(|e| e.to_string())?.value().to_string());
            }
        }
        d = d.succ_opt().ok_or("date range overflow")?;
    }
    Ok(out)
}

pub(crate) fn deletion_tombstone_storage_key(kind: TombstoneEntityKind, id: &str) -> String {
    let prefix = match kind {
        TombstoneEntityKind::Subscription => "sub",
        TombstoneEntityKind::Expense => "exp",
        TombstoneEntityKind::Category => "cat",
        TombstoneEntityKind::Currency => "cur",
        TombstoneEntityKind::Household => "hh",
        TombstoneEntityKind::PaymentMethod => "pm",
        TombstoneEntityKind::Tag => "tag",
    };
    format!("{prefix}:{id}")
}

pub(crate) fn load_deletion_tombstones(db: &Database) -> Result<Vec<DeletionTombstone>, String> {
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = match tx.open_table(T2_DELETION_TOMBSTONES) {
        Ok(t) => t,
        Err(_) => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    let iter = table.iter().map_err(|e| e.to_string())?;
    for entry in iter {
        let (_k, v) = entry.map_err(|e| e.to_string())?;
        out.push(decode_bin::<DeletionTombstone>(v.value())?);
    }
    Ok(out)
}

pub(crate) fn upsert_deletion_tombstone(db: &Database, row: &DeletionTombstone) -> Result<(), String> {
    let key = deletion_tombstone_storage_key(row.entity_kind, &row.entity_id);
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = tx.open_table(T2_DELETION_TOMBSTONES).map_err(|e| e.to_string())?;
        let payload = encode_bin(row)?;
        table.insert(key.as_str(), payload.as_slice()).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

pub(crate) fn replace_deletion_tombstones(db: &Database, rows: &[DeletionTombstone]) -> Result<(), String> {
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = tx.open_table(T2_DELETION_TOMBSTONES).map_err(|e| e.to_string())?;
        let mut keys = Vec::new();
        let iter = table.iter().map_err(|e| e.to_string())?;
        for entry in iter {
            let (k, _) = entry.map_err(|e| e.to_string())?;
            keys.push(k.value().to_string());
        }
        for key in keys {
            let _ = table.remove(key.as_str()).map_err(|e| e.to_string())?;
        }
        for row in rows {
            let key = deletion_tombstone_storage_key(row.entity_kind, &row.entity_id);
            let payload = encode_bin(row)?;
            table.insert(key.as_str(), payload.as_slice()).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

fn default_settings_doc() -> SettingsDoc {
    SettingsDoc {
        budget: 0.0,
        main_currency_id: String::new(),
        currency_update_targets: Vec::new(),
        rate_history_days: 90,
    }
}

pub(crate) fn encode_bin<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    postcard::to_stdvec(value).map_err(|e| e.to_string())
}

pub(crate) fn decode_bin<T: DeserializeOwned>(raw: &[u8]) -> Result<T, String> {
    postcard::from_bytes(raw).map_err(|e| e.to_string())
}

pub(crate) fn read_singleton_bin_typed<T>(
    db: &Database,
    table: TableDefinition<&str, &[u8]>,
    fallback: T,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = match tx.open_table(table) {
        Ok(t) => t,
        Err(_) => return Ok(fallback),
    };
    let raw = table.get(KEY_DATA).map_err(|e| e.to_string())?;
    match raw {
        Some(v) => decode_bin(v.value())
            .map_err(|e| format!("singleton decode failed (key '{}'): {}", KEY_DATA, e)),
        None => Ok(fallback),
    }
}


pub(crate) fn read_entity_table_bin_typed<T>(
    db: &Database,
    table: TableDefinition<&str, &[u8]>,
) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    let tx = db.begin_read().map_err(|e| e.to_string())?;
    let table = match tx.open_table(table) {
        Ok(t) => t,
        Err(_) => return Ok(Vec::new()),
    };
    let mut rows = Vec::new();
    let iter = table.iter().map_err(|e| e.to_string())?;
    for entry in iter {
        let (k, value) = entry.map_err(|e| e.to_string())?;
        rows.push(decode_bin::<T>(value.value())
            .map_err(|e| format!("entity decode failed (key '{}'): {}", k.value(), e))?);
    }
    Ok(rows)
}

/// Replace all rows in one table inside an existing write transaction (full table rewrite).
pub(crate) fn write_entity_table_bin_in_tx<T, F>(
    tx: &WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
    rows: &[T],
    get_id: F,
) -> Result<(), String>
where
    T: Serialize,
    F: Fn(&T) -> Option<String>,
{
    let mut table = tx.open_table(table).map_err(|e| e.to_string())?;
    let mut keys = Vec::new();
    let iter = table.iter().map_err(|e| e.to_string())?;
    for entry in iter {
        let (k, _) = entry.map_err(|e| e.to_string())?;
        keys.push(k.value().to_string());
    }
    for key in keys {
        let _ = table.remove(key.as_str()).map_err(|e| e.to_string())?;
    }
    for row in rows {
        let id = get_id(row).ok_or("entity id missing")?;
        let payload = encode_bin(row)?;
        table.insert(id.as_str(), payload.as_slice()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn write_singleton_bin_in_tx<T>(
    tx: &WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
    value: &T,
) -> Result<(), String>
where
    T: Serialize,
{
    let mut table = tx.open_table(table).map_err(|e| e.to_string())?;
    let payload = encode_bin(value)?;
    table.insert(KEY_DATA, payload.as_slice()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Atomically persist all typed entity tables plus settings and optional config (single commit).
pub(crate) fn persist_app_data_snapshot(
    db: &Database,
    doc: &AppDataDoc,
    config: Option<&AppConfigDoc>,
) -> Result<(), String> {
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    write_entity_table_bin_in_tx(&tx, T2_SUBSCRIPTIONS, &doc.subscriptions, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_EXPENSES, &doc.expenses, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_CATEGORIES, &doc.categories, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_CURRENCIES, &doc.currencies, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_HOUSEHOLD, &doc.household, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_PAYMENT_METHODS, &doc.payment_methods, |x| Some(x.id.clone()))?;
    write_entity_table_bin_in_tx(&tx, T2_TAGS, &doc.tags, |x| Some(x.id.clone()))?;
    write_singleton_bin_in_tx(&tx, T2_SETTINGS, &doc.settings)?;
    if let Some(cfg) = config {
        write_singleton_bin_in_tx(&tx, T2_CONFIG, cfg)?;
    }
    rewrite_expense_day_index_in_tx(&tx, &doc.expenses)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Per-table full replace (separate commit). Kept for tests and low-level operations; prefer [`persist_app_data_snapshot`].
#[allow(dead_code)]
pub(crate) fn write_entity_table_bin_typed<T, F>(
    db: &Database,
    table: TableDefinition<&str, &[u8]>,
    rows: &[T],
    get_id: F,
) -> Result<(), String>
where
    T: Serialize,
    F: Fn(&T) -> Option<String>,
{
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    write_entity_table_bin_in_tx(&tx, table, rows, get_id)?;
    tx.commit().map_err(|e| e.to_string())
}

pub(crate) fn write_singleton_bin_typed<T>(
    db: &Database,
    table: TableDefinition<&str, &[u8]>,
    value: &T,
) -> Result<(), String>
where
    T: Serialize,
{
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    write_singleton_bin_in_tx(&tx, table, value)?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn load_app_data_typed(db: &Database) -> Result<(AppDataDoc, AppConfigDoc), String> {
    let subscriptions: Vec<SubscriptionDoc> = read_entity_table_bin_typed(db, T2_SUBSCRIPTIONS)?;
    let expenses: Vec<ExpenseDoc> = read_entity_table_bin_typed(db, T2_EXPENSES)?;
    let categories: Vec<CategoryDoc> = read_entity_table_bin_typed(db, T2_CATEGORIES)?;
    let currencies: Vec<CurrencyDoc> = read_entity_table_bin_typed(db, T2_CURRENCIES)?;
    let household: Vec<HouseholdMemberDoc> = read_entity_table_bin_typed(db, T2_HOUSEHOLD)?;
    let payment_methods: Vec<PaymentMethodDoc> = read_entity_table_bin_typed(db, T2_PAYMENT_METHODS)?;
    let tags: Vec<TagDoc> = read_entity_table_bin_typed(db, T2_TAGS)?;
    let settings: SettingsDoc = read_singleton_bin_typed(db, T2_SETTINGS, default_settings_doc())?;
    let config: AppConfigDoc = read_singleton_bin_typed(db, T2_CONFIG, AppConfigDoc::default())?;
    Ok((
        AppDataDoc {
            subscriptions,
            expenses,
            categories,
            currencies,
            household,
            payment_methods,
            tags,
            settings,
        },
        config,
    ))
}

pub fn save_app_data_typed(db: &Database, doc: &AppDataDoc, cfg: &AppConfigDoc) -> Result<(), String> {
    persist_app_data_snapshot(db, doc, Some(cfg))
}

#[allow(dead_code)]
fn clear_table(db: &Database, table: TableDefinition<&str, &[u8]>) -> Result<(), String> {
    let tx = db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut t = tx.open_table(table).map_err(|e| e.to_string())?;
        let mut keys = Vec::new();
        let iter = t.iter().map_err(|e| e.to_string())?;
        for entry in iter {
            let (k, _) = entry.map_err(|e| e.to_string())?;
            keys.push(k.value().to_string());
        }
        for key in keys {
            let _ = t.remove(key.as_str()).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn clear_all_typed_data(db: &Database) -> Result<(), String> {
    clear_table(db, T2_SUBSCRIPTIONS)?;
    clear_table(db, T2_EXPENSES)?;
    clear_table(db, T2_CATEGORIES)?;
    clear_table(db, T2_CURRENCIES)?;
    clear_table(db, T2_HOUSEHOLD)?;
    clear_table(db, T2_PAYMENT_METHODS)?;
    clear_table(db, T2_TAGS)?;
    clear_table(db, T2_SETTINGS)?;
    clear_table(db, T2_CONFIG)?;
    clear_table(db, T2_RATE_HISTORY)?;
    clear_table(db, T2_DELETION_TOMBSTONES)?;
    Ok(())
}

pub fn load_rate_history(db: &Database) -> Result<String, String> {
    let v: std::collections::HashMap<String, Vec<crate::RatePoint>> =
        read_singleton_bin_typed(db, T2_RATE_HISTORY, std::collections::HashMap::new())?;
    serde_json::to_string(&v).map_err(|e| e.to_string())
}

pub fn save_rate_history(db: &Database, raw_json: &str) -> Result<(), String> {
    let v: std::collections::HashMap<String, Vec<crate::RatePoint>> =
        serde_json::from_str(raw_json).map_err(|e| e.to_string())?;
    write_singleton_bin_typed(db, T2_RATE_HISTORY, &v)
}

impl AppStateInner {
    pub fn apply_snapshot_typed(&mut self, data: &AppDataDoc) -> Result<(), String> {
        let doc = crate::state_tables::touch_app_data_for_apply_snapshot(data)?;
        persist_app_data_snapshot(self.db.as_ref(), &doc, None)?;
        self.app_data = doc;
        Ok(())
    }

    pub fn apply_snapshot_typed_with_config(
        &mut self,
        data: &AppDataDoc,
        config: &AppConfigDoc,
    ) -> Result<(), String> {
        let doc = crate::state_tables::touch_app_data_for_apply_snapshot(data)?;
        persist_app_data_snapshot(self.db.as_ref(), &doc, Some(config))?;
        self.app_data = doc;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn reload_from_db(&mut self) -> Result<(), String> {
        let (doc, _) = load_app_data_typed(self.db.as_ref())?;
        self.app_data = doc;
        Ok(())
    }

    pub fn doc(&self) -> Result<crate::AppDataDoc, String> {
        Ok(self.app_data.clone())
    }

    pub fn redb_get(&self, key: &str) -> Result<Option<String>, String> {
        let tx = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = tx.open_table(crate::KV_TABLE).map_err(|e| e.to_string())?;
        let maybe = table.get(key).map_err(|e| e.to_string())?;
        Ok(maybe.map(|v| v.value().to_string()))
    }

    pub fn redb_set(&self, key: &str, value: &str) -> Result<(), String> {
        let tx = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = tx.open_table(crate::KV_TABLE).map_err(|e| e.to_string())?;
            table.insert(key, value).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn redb_delete(&self, key: &str) -> Result<(), String> {
        let tx = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = tx.open_table(crate::KV_TABLE).map_err(|e| e.to_string())?;
            let _ = table.remove(key).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_roundtrip_keeps_restart_sensitive_fields() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let cfg = crate::test_support::default_config();

        save_app_data_typed(&db, &before, &cfg).expect("save typed");
        let (after, loaded_cfg) = load_app_data_typed(&db).expect("load typed");

        assert!(loaded_cfg.initialized, "config initialized should persist");
        crate::test_support::assert_core_invariants(&before, &after);
    }

    #[test]
    #[ignore = "known-failing risk-id:partial-write-multi-table"]
    fn known_failing_partial_write_can_break_consistency_across_tables() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let mut before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let cfg = crate::test_support::default_config();
        save_app_data_typed(&db, &before, &cfg).expect("save initial");

        // Simulate a mid-flight write where one table is replaced and process dies.
        before.subscriptions[0].url = "https://example.com/new-url".to_string();
        before.expenses[0].notes = "expense-updated-in-same-snapshot".to_string();
        write_entity_table_bin_typed(&db, T2_SUBSCRIPTIONS, &before.subscriptions, |x| Some(x.id.clone()))
            .expect("write subscriptions only");

        let (after, _) = load_app_data_typed(&db).expect("load typed");
        // Intentionally strict expectation to expose the architectural risk as a red test when run with --ignored.
        assert_eq!(
            before.expenses[0].notes,
            after.expenses[0].notes,
            "known risk: multi-table partial write leaves mixed-state snapshot after restart"
        );
    }

    #[test]
    fn table_get_by_id_reads_single_entity_without_full_scan_in_callers() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let cfg = crate::test_support::default_config();
        save_app_data_typed(&db, &doc, &cfg).expect("save typed");

        let mut state = AppStateInner {
            db: std::sync::Arc::new(db),
            app_data: doc.clone(),
        };
        let target_id = doc.subscriptions[0].id.clone();
        let loaded = state
            .table_get_by_id_typed::<crate::models::SubscriptionDoc>(EntityTable::Subscriptions, &target_id)
            .expect("read by id")
            .expect("entity exists");
        assert_eq!(loaded.id, target_id, "single-entity read should return requested id");

        let missing = state
            .table_get_by_id_typed::<crate::models::SubscriptionDoc>(EntityTable::Subscriptions, "missing-id")
            .expect("read missing");
        assert!(missing.is_none(), "missing entity should return None");

        // keep state mutable usage explicit for future command-side point updates
        state.app_data.subscriptions.clear();
    }
}
