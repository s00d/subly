use redb::{ReadableDatabase, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{AppError, AppResult};
use crate::models::{ExpenseDoc, SubscriptionDoc, TombstoneEntityKind};
use crate::state::{
    decode_bin, encode_bin, read_singleton_bin_typed, upsert_deletion_tombstone, AppStateInner, T2_CATEGORIES,
    T2_CONFIG, T2_CURRENCIES, T2_EXPENSES, T2_HOUSEHOLD, T2_PAYMENT_METHODS, T2_SETTINGS, T2_SUBSCRIPTIONS, T2_TAGS,
};

/// Touch `updatedAt` on subscriptions and expenses (same as per-table replace) for a full snapshot apply.
pub(crate) fn touch_subscription_doc(mut row: SubscriptionDoc) -> SubscriptionDoc {
    row.updated_at = chrono::Utc::now().timestamp_millis();
    row
}

pub(crate) fn touch_expense_doc(mut row: ExpenseDoc) -> ExpenseDoc {
    row.updated_at = chrono::Utc::now().timestamp_millis();
    row
}

pub(crate) fn touch_app_data_for_apply_snapshot(data: &crate::AppDataDoc) -> AppResult<crate::AppDataDoc> {
    let subscriptions = data.subscriptions.iter().cloned().map(touch_subscription_doc).collect::<Vec<_>>();
    let expenses = data.expenses.iter().cloned().map(touch_expense_doc).collect::<Vec<_>>();
    Ok(crate::AppDataDoc {
        subscriptions,
        expenses,
        categories: data.categories.clone(),
        currencies: data.currencies.clone(),
        household: data.household.clone(),
        payment_methods: data.payment_methods.clone(),
        tags: data.tags.clone(),
        settings: data.settings.clone(),
    })
}

#[derive(Clone, Copy)]
pub enum EntityTable {
    Subscriptions,
    Expenses,
    Categories,
    Currencies,
    Household,
    PaymentMethods,
    Tags,
}

impl EntityTable {
    pub(crate) fn tombstone_kind(self) -> TombstoneEntityKind {
        match self {
            EntityTable::Subscriptions => TombstoneEntityKind::Subscription,
            EntityTable::Expenses => TombstoneEntityKind::Expense,
            EntityTable::Categories => TombstoneEntityKind::Category,
            EntityTable::Currencies => TombstoneEntityKind::Currency,
            EntityTable::Household => TombstoneEntityKind::Household,
            EntityTable::PaymentMethods => TombstoneEntityKind::PaymentMethod,
            EntityTable::Tags => TombstoneEntityKind::Tag,
        }
    }
}

impl AppStateInner {
    fn is_timestamped_entity_table(table: EntityTable) -> bool {
        matches!(table, EntityTable::Subscriptions | EntityTable::Expenses)
    }

    pub(crate) fn table_def(table: EntityTable) -> TableDefinition<'static, &'static str, &'static [u8]> {
        match table {
            EntityTable::Subscriptions => T2_SUBSCRIPTIONS,
            EntityTable::Expenses => T2_EXPENSES,
            EntityTable::Categories => T2_CATEGORIES,
            EntityTable::Currencies => T2_CURRENCIES,
            EntityTable::Household => T2_HOUSEHOLD,
            EntityTable::PaymentMethods => T2_PAYMENT_METHODS,
            EntityTable::Tags => T2_TAGS,
        }
    }

    pub fn table_list_typed<T>(&self, table: EntityTable) -> AppResult<Vec<T>>
    where
        T: Clone + DeserializeOwned,
    {
        match table {
            EntityTable::Subscriptions => convert_vec(&self.app_data.subscriptions),
            EntityTable::Expenses => convert_vec(&self.app_data.expenses),
            EntityTable::Categories => convert_vec(&self.app_data.categories),
            EntityTable::Currencies => convert_vec(&self.app_data.currencies),
            EntityTable::Household => convert_vec(&self.app_data.household),
            EntityTable::PaymentMethods => convert_vec(&self.app_data.payment_methods),
            EntityTable::Tags => convert_vec(&self.app_data.tags),
        }
    }

    pub fn table_get_by_id_typed<T>(&self, table: EntityTable, id: &str) -> AppResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let tx = self.db.begin_read().map_err(AppError::from)?;
        let table_def = Self::table_def(table);
        let t = tx.open_table(table_def).map_err(AppError::from)?;
        let maybe = t.get(id).map_err(AppError::from)?;
        match maybe {
            Some(raw) => Ok(Some(decode_bin::<T>(raw.value())?)),
            None => Ok(None),
        }
    }

    pub fn table_get_expense_by_id(&self, id: &str) -> AppResult<Option<crate::models::ExpenseDoc>> {
        let tx = self.db.begin_read().map_err(AppError::from)?;
        let t = tx.open_table(Self::table_def(EntityTable::Expenses)).map_err(AppError::from)?;
        let maybe = t.get(id).map_err(AppError::from)?;
        match maybe {
            Some(raw) => Ok(Some(decode_bin::<crate::models::ExpenseDoc>(raw.value())?)),
            None => Ok(None),
        }
    }

    pub fn table_upsert_typed<T>(&mut self, table: EntityTable, row: &T, id: &str) -> AppResult<()>
    where
        T: Clone + Serialize + DeserializeOwned,
    {
        let row_to_write = if Self::is_timestamped_entity_table(table) {
            touch_timestamped_row(table, row)?
        } else {
            row.clone()
        };
        let tx = self.db.begin_write().map_err(AppError::from)?;
        {
            let mut t = tx.open_table(Self::table_def(table)).map_err(AppError::from)?;
            let payload = encode_bin(&row_to_write)?;
            t.insert(id, payload.as_slice()).map_err(AppError::from)?;
        }
        tx.commit().map_err(AppError::from)?;
        self.patch_cache_upsert_typed(table, &row_to_write, id)
    }

    pub fn table_delete_by_id(&mut self, table: EntityTable, id: &str) -> AppResult<()> {
        let tx = self.db.begin_write().map_err(AppError::from)?;
        {
            let mut t = tx.open_table(Self::table_def(table)).map_err(AppError::from)?;
            let _ = t.remove(id).map_err(AppError::from)?;
        }
        tx.commit().map_err(AppError::from)?;
        self.patch_cache_delete(table, id);
        let device_id = crate::commands::sync::load_sync_config()
            .map(|c| c.device_id)
            .unwrap_or_default();
        let deleted_at = chrono::Utc::now().timestamp_millis();
        upsert_deletion_tombstone(
            self.db.as_ref(),
            &crate::models::DeletionTombstone {
                entity_kind: table.tombstone_kind(),
                entity_id: id.to_string(),
                deleted_at,
                device_id,
            },
        )?;
        Ok(())
    }

    pub fn settings_typed<T>(&self) -> AppResult<T>
    where
        T: Default + DeserializeOwned,
    {
        read_singleton_bin_typed(self.db.as_ref(), T2_SETTINGS, T::default())
    }

    pub fn config_typed<T>(&self) -> AppResult<T>
    where
        T: Default + DeserializeOwned,
    {
        read_singleton_bin_typed(self.db.as_ref(), T2_CONFIG, T::default())
    }

    fn patch_cache_upsert_typed<T>(&mut self, table: EntityTable, row: &T, id: &str) -> AppResult<()>
    where
        T: Clone + Serialize + DeserializeOwned,
    {
        match table {
            EntityTable::Subscriptions => upsert_doc_typed(&mut self.app_data.subscriptions, row, id)?,
            EntityTable::Expenses => upsert_doc_typed(&mut self.app_data.expenses, row, id)?,
            EntityTable::Categories => upsert_doc_typed(&mut self.app_data.categories, row, id)?,
            EntityTable::Currencies => upsert_doc_typed(&mut self.app_data.currencies, row, id)?,
            EntityTable::Household => upsert_doc_typed(&mut self.app_data.household, row, id)?,
            EntityTable::PaymentMethods => upsert_doc_typed(&mut self.app_data.payment_methods, row, id)?,
            EntityTable::Tags => upsert_doc_typed(&mut self.app_data.tags, row, id)?,
        }
        Ok(())
    }

    fn patch_cache_delete(&mut self, table: EntityTable, id: &str) {
        match table {
            EntityTable::Subscriptions => self.app_data.subscriptions.retain(|x| x.id != id),
            EntityTable::Expenses => self.app_data.expenses.retain(|x| x.id != id),
            EntityTable::Categories => self.app_data.categories.retain(|x| x.id != id),
            EntityTable::Currencies => self.app_data.currencies.retain(|x| x.id != id),
            EntityTable::Household => self.app_data.household.retain(|x| x.id != id),
            EntityTable::PaymentMethods => self.app_data.payment_methods.retain(|x| x.id != id),
            EntityTable::Tags => self.app_data.tags.retain(|x| x.id != id),
        }
    }
}

fn upsert_doc_typed<T, U>(target: &mut Vec<T>, row: &U, id: &str) -> AppResult<()>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
    U: Serialize,
{
    let parsed: T = serde_json::from_value(serde_json::to_value(row).map_err(AppError::from)?).map_err(AppError::from)?;
    if let Some(pos) = target
        .iter()
        .position(|x| convert_id(x).ok().as_deref() == Some(id))
    {
        target[pos] = parsed;
    } else {
        target.push(parsed);
    }
    Ok(())
}

fn convert_vec<TSrc, TDst>(src: &[TSrc]) -> AppResult<Vec<TDst>>
where
    TSrc: Serialize,
    TDst: DeserializeOwned,
{
    src.iter()
        .map(|row| {
            let decoded: TDst =
                serde_json::from_value(serde_json::to_value(row).map_err(AppError::from)?).map_err(AppError::from)?;
            Ok(decoded)
        })
        .collect()
}

#[derive(serde::Deserialize)]
struct EntityIdOnly {
    id: String,
}

fn convert_id<T: Serialize>(value: &T) -> AppResult<String> {
    let id_only: EntityIdOnly =
        serde_json::from_value(serde_json::to_value(value).map_err(AppError::from)?).map_err(AppError::from)?;
    if id_only.id.is_empty() {
        return Err(AppError::EntityIdMissing);
    }
    Ok(id_only.id)
}

fn touch_timestamped_row<T>(table: EntityTable, row: &T) -> AppResult<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    match table {
        EntityTable::Subscriptions => {
            let parsed: SubscriptionDoc =
                serde_json::from_value(serde_json::to_value(row).map_err(AppError::from)?).map_err(AppError::from)?;
            let touched = touch_subscription_doc(parsed);
            serde_json::from_value(serde_json::to_value(&touched).map_err(AppError::from)?).map_err(AppError::from)
        }
        EntityTable::Expenses => {
            let parsed: ExpenseDoc =
                serde_json::from_value(serde_json::to_value(row).map_err(AppError::from)?).map_err(AppError::from)?;
            let touched = touch_expense_doc(parsed);
            serde_json::from_value(serde_json::to_value(&touched).map_err(AppError::from)?).map_err(AppError::from)
        }
        _ => Ok(row.clone()),
    }
}
