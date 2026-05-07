use tauri::State;
use crate::AppState;
use crate::models::{CatalogsLoadDto, CatalogsUsageSummaryDto, CategoryDoc, CurrencyDoc, CurrencyRateUpdateDto, ExpenseDoc, HouseholdMemberDoc, PaymentMethodDoc, SubscriptionDoc, TagDoc};
use crate::state::EntityTable;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SortableRowDto {
    #[serde(default)]
    sort_order: i64,
}

#[tauri::command]
pub fn catalogs_load(state: State<'_, AppState>) -> Result<CatalogsLoadDto, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    Ok(CatalogsLoadDto {
        categories: guard.table_list_typed(EntityTable::Categories)?,
        currencies: guard.table_list_typed(EntityTable::Currencies)?,
        household: guard.table_list_typed(EntityTable::Household)?,
        payment_methods: guard.table_list_typed(EntityTable::PaymentMethods)?,
        tags: guard.table_list_typed(EntityTable::Tags)?,
    })
}

#[tauri::command]
pub fn catalogs_usage_summary(state: State<'_, AppState>) -> Result<CatalogsUsageSummaryDto, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;

    let mut category_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut currency_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut payment_method_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut tag_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    for sub in &subscriptions {
        if !sub.category_id.is_empty() {
            *category_usage.entry(sub.category_id.clone()).or_insert(0) += 1;
        }
        if !sub.currency_id.is_empty() {
            *currency_usage.entry(sub.currency_id.clone()).or_insert(0) += 1;
        }
        if !sub.payment_method_id.is_empty() {
            *payment_method_usage.entry(sub.payment_method_id.clone()).or_insert(0) += 1;
        }
    }

    for exp in &expenses {
        if !exp.category_id.is_empty() {
            *category_usage.entry(exp.category_id.clone()).or_insert(0) += 1;
        }
        if !exp.currency_id.is_empty() {
            *currency_usage.entry(exp.currency_id.clone()).or_insert(0) += 1;
        }
        for tag in &exp.tags {
            if !tag.is_empty() {
                *tag_usage.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    Ok(CatalogsUsageSummaryDto {
        category_usage,
        currency_usage,
        payment_method_usage,
        tag_usage,
    })
}

#[tauri::command]
pub fn catalogs_upsert_category(state: State<'_, AppState>, category: CategoryDoc) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_upsert_typed(EntityTable::Categories, &category, &category.id)
}
#[tauri::command]
pub fn catalogs_delete_category(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_delete_by_id(EntityTable::Categories, &id)
}
#[tauri::command]
pub fn catalogs_upsert_currency(state: State<'_, AppState>, currency: CurrencyDoc) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_upsert_typed(EntityTable::Currencies, &currency, &currency.id)
}
#[tauri::command]
pub fn catalogs_delete_currency(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_delete_by_id(EntityTable::Currencies, &id)
}
#[tauri::command]
pub fn catalogs_update_currency_rates(state: State<'_, AppState>, updates: Vec<CurrencyRateUpdateDto>) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let mut arr: Vec<CurrencyDoc> = guard.table_list_typed(EntityTable::Currencies)?;
    let mut touched_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for u in updates {
        if let Some(c) = arr.iter_mut().find(|c| c.id == u.id) {
            c.rate = u.rate;
            touched_ids.insert(c.id.clone());
        }
    }
    for c in &arr {
        if touched_ids.contains(&c.id) {
            guard.table_upsert_typed(EntityTable::Currencies, c, &c.id)?;
        }
    }
    Ok(())
}
#[tauri::command]
pub fn catalogs_upsert_household_member(state: State<'_, AppState>, household_member: HouseholdMemberDoc) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_upsert_typed(EntityTable::Household, &household_member, &household_member.id)
}
#[tauri::command]
pub fn catalogs_delete_household_member(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_delete_by_id(EntityTable::Household, &id)
}
#[tauri::command]
pub fn catalogs_upsert_payment_method(state: State<'_, AppState>, payment_method: PaymentMethodDoc) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_upsert_typed(EntityTable::PaymentMethods, &payment_method, &payment_method.id)
}
#[tauri::command]
pub fn catalogs_delete_payment_method(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_delete_by_id(EntityTable::PaymentMethods, &id)
}
#[tauri::command]
pub fn catalogs_upsert_tag(state: State<'_, AppState>, tag: TagDoc) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_upsert_typed(EntityTable::Tags, &tag, &tag.id)
}
#[tauri::command]
pub fn catalogs_delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    guard.table_delete_by_id(EntityTable::Tags, &id)
}
#[tauri::command]
pub fn catalogs_max_sort_order(state: State<'_, AppState>, table: String) -> Result<i64, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let max = match table.as_str() {
        "categories" => guard
            .table_list_typed::<SortableRowDto>(EntityTable::Categories)?
            .iter()
            .map(|x| x.sort_order)
            .max()
            .unwrap_or(0),
        "currencies" => guard
            .table_list_typed::<SortableRowDto>(EntityTable::Currencies)?
            .iter()
            .map(|x| x.sort_order)
            .max()
            .unwrap_or(0),
        "householdMembers" => guard
            .table_list_typed::<SortableRowDto>(EntityTable::Household)?
            .iter()
            .map(|x| x.sort_order)
            .max()
            .unwrap_or(0),
        "paymentMethods" => guard
            .table_list_typed::<SortableRowDto>(EntityTable::PaymentMethods)?
            .iter()
            .map(|x| x.sort_order)
            .max()
            .unwrap_or(0),
        "tags" => guard
            .table_list_typed::<SortableRowDto>(EntityTable::Tags)?
            .iter()
            .map(|x| x.sort_order)
            .max()
            .unwrap_or(0),
        _ => 0,
    };
    Ok(max)
}
