use tauri::{Emitter, State};
use crate::AppState;
use crate::errors::AppError;
use crate::models::{
    CatalogsLoadDto, CatalogsUsageSummaryDto, CategoryDoc, CurrencyDoc, CurrencyRateUpdateDto,
    ExpenseDoc, HouseholdMemberDoc, PaymentMethodDoc, SubscriptionDoc, TagDoc,
};
use crate::state::EntityTable;

const PROTECTED_CATEGORY_ID: &str = "cat-1";
const FALLBACK_CATEGORY_ID: &str = "cat-1";
const FALLBACK_CURRENCY_ID: &str = "cur-1";
const FALLBACK_HOUSEHOLD_ID: &str = "hm-1";
const CONFIG_SETTINGS_KEY: &str = "config:settings";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SortableRowDto {
    #[serde(default)]
    sort_order: i64,
}

#[derive(Debug, Clone)]
struct CatalogDefaults {
    main_currency_id: String,
    default_category_id: String,
    default_payment_method_id: String,
}

fn emit_app_data_changed(app: &tauri::AppHandle) {
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "appData", "action": "catalog_delete" }),
    );
}

fn parse_config_settings(raw: &str) -> Option<serde_json::Value> {
    let parsed = serde_json::from_str::<serde_json::Value>(raw).ok()?;
    match parsed {
        serde_json::Value::String(inner) => serde_json::from_str(&inner).ok(),
        other => Some(other),
    }
}

fn json_string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Category id for rows/settings after removing `deleting_id`.
fn resolve_category_replacement(
    categories: &[CategoryDoc],
    deleting_id: &str,
    defaults: &CatalogDefaults,
) -> Result<String, AppError> {
    let preferred = if defaults.default_category_id == deleting_id {
        FALLBACK_CATEGORY_ID
    } else {
        defaults.default_category_id.as_str()
    };

    if preferred != deleting_id && categories.iter().any(|c| c.id == preferred) {
        return Ok(preferred.to_string());
    }

    if deleting_id != FALLBACK_CATEGORY_ID
        && categories.iter().any(|c| c.id == FALLBACK_CATEGORY_ID)
    {
        return Ok(FALLBACK_CATEGORY_ID.to_string());
    }

    categories
        .iter()
        .find(|c| c.id != deleting_id)
        .map(|c| c.id.clone())
        .ok_or_else(|| AppError::from("No category available for reassignment."))
}

fn load_catalog_defaults(
    guard: &crate::state::AppStateInner,
) -> Result<CatalogDefaults, AppError> {
    if let Some(raw) = guard.redb_get(CONFIG_SETTINGS_KEY)? {
        if let Some(settings) = parse_config_settings(&raw) {
            return Ok(CatalogDefaults {
                main_currency_id: json_string_field(&settings, "mainCurrencyId")
                    .unwrap_or_else(|| FALLBACK_CURRENCY_ID.to_string()),
                default_category_id: json_string_field(&settings, "defaultCategoryId")
                    .unwrap_or_else(|| FALLBACK_CATEGORY_ID.to_string()),
                default_payment_method_id: json_string_field(&settings, "defaultPaymentMethodId")
                    .unwrap_or_else(|| "pm-1".to_string()),
            });
        }
    }

    let categories: Vec<CategoryDoc> = guard.table_list_typed(EntityTable::Categories)?;
    let currencies: Vec<CurrencyDoc> = guard.table_list_typed(EntityTable::Currencies)?;
    let payment_methods: Vec<PaymentMethodDoc> = guard.table_list_typed(EntityTable::PaymentMethods)?;

    Ok(CatalogDefaults {
        main_currency_id: currencies
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| FALLBACK_CURRENCY_ID.to_string()),
        default_category_id: categories
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| FALLBACK_CATEGORY_ID.to_string()),
        default_payment_method_id: payment_methods
            .first()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "pm-1".to_string()),
    })
}

fn save_ui_settings(
    guard: &crate::state::AppStateInner,
    settings: &serde_json::Value,
) -> Result<(), AppError> {
    let raw = serde_json::to_string(settings).map_err(|e| e.to_string())?;
    guard.redb_set(CONFIG_SETTINGS_KEY, &raw)
}

fn patch_ui_settings(
    guard: &crate::state::AppStateInner,
    patch: impl FnOnce(&mut serde_json::Value),
) -> Result<(), AppError> {
    let mut settings = if let Some(raw) = guard.redb_get(CONFIG_SETTINGS_KEY)? {
        parse_config_settings(&raw).unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    patch(&mut settings);
    save_ui_settings(guard, &settings)
}

fn count_catalog_usage(
    subscriptions: &[SubscriptionDoc],
    expenses: &[ExpenseDoc],
) -> CatalogsUsageSummaryDto {
    let mut category_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut currency_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut payment_method_usage: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut tag_usage: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    for sub in subscriptions {
        if !sub.category_id.is_empty() {
            *category_usage.entry(sub.category_id.clone()).or_insert(0) += 1;
        }
        if !sub.currency_id.is_empty() {
            *currency_usage.entry(sub.currency_id.clone()).or_insert(0) += 1;
        }
        if !sub.payment_method_id.is_empty() {
            *payment_method_usage
                .entry(sub.payment_method_id.clone())
                .or_insert(0) += 1;
        }
        for tag in &sub.tags {
            if !tag.is_empty() {
                *tag_usage.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    for exp in expenses {
        if !exp.category_id.is_empty() {
            *category_usage.entry(exp.category_id.clone()).or_insert(0) += 1;
        }
        if !exp.currency_id.is_empty() {
            *currency_usage.entry(exp.currency_id.clone()).or_insert(0) += 1;
        }
        if !exp.payment_method_id.is_empty() {
            *payment_method_usage
                .entry(exp.payment_method_id.clone())
                .or_insert(0) += 1;
        }
        for tag in &exp.tags {
            if !tag.is_empty() {
                *tag_usage.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    CatalogsUsageSummaryDto {
        category_usage,
        currency_usage,
        payment_method_usage,
        tag_usage,
    }
}

#[tauri::command]
pub fn catalogs_load(state: State<'_, AppState>) -> Result<CatalogsLoadDto, AppError> {
    let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    Ok(CatalogsLoadDto {
        categories: guard.table_list_typed(EntityTable::Categories)?,
        currencies: guard.table_list_typed(EntityTable::Currencies)?,
        household: guard.table_list_typed(EntityTable::Household)?,
        payment_methods: guard.table_list_typed(EntityTable::PaymentMethods)?,
        tags: guard.table_list_typed(EntityTable::Tags)?,
    })
}

#[tauri::command]
pub fn catalogs_usage_summary(state: State<'_, AppState>) -> Result<CatalogsUsageSummaryDto, AppError> {
    let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    Ok(count_catalog_usage(&subscriptions, &expenses))
}

#[tauri::command]
pub fn catalogs_upsert_category(state: State<'_, AppState>, category: CategoryDoc) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    guard.table_upsert_typed(EntityTable::Categories, &category, &category.id)
}

#[tauri::command]
pub fn catalogs_delete_category(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    if id == PROTECTED_CATEGORY_ID {
        return Err(AppError::from("This category cannot be deleted."));
    }

    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let defaults = load_catalog_defaults(&guard)?;
    let categories: Vec<CategoryDoc> = guard.table_list_typed(EntityTable::Categories)?;
    let replacement = resolve_category_replacement(&categories, &id, &defaults)?;

    let mut subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let mut expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    let mut touched_subs = false;
    let mut touched_expenses = false;

    for sub in &mut subscriptions {
        if sub.category_id == id {
            sub.category_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            touched_subs = true;
        }
    }
    for exp in &mut expenses {
        if exp.category_id == id {
            exp.category_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Expenses, exp, &exp.id)?;
            touched_expenses = true;
        }
    }

    if defaults.default_category_id == id {
        patch_ui_settings(&guard, |settings| {
            settings["defaultCategoryId"] = serde_json::Value::String(replacement.clone());
        })?;
        let _ = app.emit(
            "app:data-changed",
            serde_json::json!({ "entity": "settings", "action": "catalog_delete" }),
        );
    }

    guard.table_delete_by_id(EntityTable::Categories, &id)?;
    drop(guard);
    if touched_subs || touched_expenses {
        emit_app_data_changed(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn catalogs_upsert_currency(state: State<'_, AppState>, currency: CurrencyDoc) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    guard.table_upsert_typed(EntityTable::Currencies, &currency, &currency.id)
}

#[tauri::command]
pub fn catalogs_delete_currency(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let defaults = load_catalog_defaults(&guard)?;

    if id == defaults.main_currency_id {
        return Err(AppError::from(
            "You cannot remove the main currency or a currency that is still in use.",
        ));
    }

    let currencies: Vec<CurrencyDoc> = guard.table_list_typed(EntityTable::Currencies)?;
    if currencies.len() <= 1 {
        return Err(AppError::from("You cannot delete the only currency."));
    }

    let replacement = defaults.main_currency_id.clone();
    let mut subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let mut expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    let mut touched = false;

    for sub in &mut subscriptions {
        if sub.currency_id == id {
            sub.currency_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            touched = true;
        }
    }
    for exp in &mut expenses {
        if exp.currency_id == id {
            exp.currency_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Expenses, exp, &exp.id)?;
            touched = true;
        }
    }

    patch_ui_settings(&guard, |settings| {
        if let Some(targets) = settings
            .get_mut("currencyUpdateTargets")
            .and_then(|v| v.as_array_mut())
        {
            targets.retain(|v| v.as_str() != Some(id.as_str()));
        }
    })?;

    guard.table_delete_by_id(EntityTable::Currencies, &id)?;
    drop(guard);
    if touched {
        emit_app_data_changed(&app);
    }
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "settings", "action": "catalog_delete" }),
    );
    Ok(())
}

#[tauri::command]
pub fn catalogs_update_currency_rates(
    state: State<'_, AppState>,
    updates: Vec<CurrencyRateUpdateDto>,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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
pub fn catalogs_upsert_household_member(
    state: State<'_, AppState>,
    household_member: HouseholdMemberDoc,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    guard.table_upsert_typed(EntityTable::Household, &household_member, &household_member.id)
}

#[tauri::command]
pub fn catalogs_delete_household_member(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let members: Vec<HouseholdMemberDoc> = guard.table_list_typed(EntityTable::Household)?;
    if members.len() <= 1 {
        return Err(AppError::from("At least one household member must remain."));
    }

    let replacement = members
        .iter()
        .find(|m| m.id != id)
        .map(|m| m.id.clone())
        .unwrap_or_else(|| FALLBACK_HOUSEHOLD_ID.to_string());

    let mut subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let mut expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    let mut touched = false;

    for sub in &mut subscriptions {
        if sub.payer_user_id == id {
            sub.payer_user_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            touched = true;
        }
    }
    for exp in &mut expenses {
        if exp.payer_user_id == id {
            exp.payer_user_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Expenses, exp, &exp.id)?;
            touched = true;
        }
    }

    guard.table_delete_by_id(EntityTable::Household, &id)?;
    drop(guard);
    if touched {
        emit_app_data_changed(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn catalogs_upsert_payment_method(
    state: State<'_, AppState>,
    payment_method: PaymentMethodDoc,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    guard.table_upsert_typed(EntityTable::PaymentMethods, &payment_method, &payment_method.id)
}

#[tauri::command]
pub fn catalogs_delete_payment_method(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let payment_methods: Vec<PaymentMethodDoc> = guard.table_list_typed(EntityTable::PaymentMethods)?;
    if payment_methods.len() <= 1 {
        return Err(AppError::from("You cannot delete the only payment method."));
    }

    let defaults = load_catalog_defaults(&guard)?;
    let replacement = payment_methods
        .iter()
        .find(|p| p.id != id)
        .map(|p| p.id.clone())
        .unwrap_or_else(|| {
            if defaults.default_payment_method_id != id {
                defaults.default_payment_method_id.clone()
            } else {
                "pm-1".to_string()
            }
        });

    let mut subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let mut expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    let mut touched = false;

    for sub in &mut subscriptions {
        if sub.payment_method_id == id {
            sub.payment_method_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            touched = true;
        }
    }
    for exp in &mut expenses {
        if exp.payment_method_id == id {
            exp.payment_method_id = replacement.clone();
            guard.table_upsert_typed(EntityTable::Expenses, exp, &exp.id)?;
            touched = true;
        }
    }

    if defaults.default_payment_method_id == id {
        patch_ui_settings(&guard, |settings| {
            settings["defaultPaymentMethodId"] =
                serde_json::Value::String(replacement.clone());
        })?;
        let _ = app.emit(
            "app:data-changed",
            serde_json::json!({ "entity": "settings", "action": "catalog_delete" }),
        );
    }

    guard.table_delete_by_id(EntityTable::PaymentMethods, &id)?;
    drop(guard);
    if touched {
        emit_app_data_changed(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn catalogs_upsert_tag(state: State<'_, AppState>, tag: TagDoc) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    guard.table_upsert_typed(EntityTable::Tags, &tag, &tag.id)
}

#[tauri::command]
pub fn catalogs_delete_tag(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let mut guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
    let tags: Vec<TagDoc> = guard.table_list_typed(EntityTable::Tags)?;
    let Some(tag) = tags.iter().find(|t| t.id == id) else {
        return Ok(());
    };
    let tag_name = tag.name.clone();
    if tag_name.is_empty() {
        guard.table_delete_by_id(EntityTable::Tags, &id)?;
        return Ok(());
    }

    let mut subscriptions: Vec<SubscriptionDoc> = guard.table_list_typed(EntityTable::Subscriptions)?;
    let mut expenses: Vec<ExpenseDoc> = guard.table_list_typed(EntityTable::Expenses)?;
    let mut touched = false;

    for sub in &mut subscriptions {
        let before = sub.tags.len();
        sub.tags.retain(|t| t != &tag_name);
        if sub.tags.len() != before {
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            touched = true;
        }
    }
    for exp in &mut expenses {
        let before = exp.tags.len();
        exp.tags.retain(|t| t != &tag_name);
        if exp.tags.len() != before {
            guard.table_upsert_typed(EntityTable::Expenses, exp, &exp.id)?;
            touched = true;
        }
    }

    guard.table_delete_by_id(EntityTable::Tags, &id)?;
    drop(guard);
    if touched {
        emit_app_data_changed(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn catalogs_max_sort_order(state: State<'_, AppState>, table: String) -> Result<i64, AppError> {
    let guard = state.lock().map_err(|_| AppError::StateLockPoisoned)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn category(id: &str) -> CategoryDoc {
        CategoryDoc {
            id: id.to_string(),
            name: id.to_string(),
            icon: String::new(),
            sort_order: 0,
            i18n_key: String::new(),
        }
    }

    #[test]
    fn resolve_category_replacement_prefers_valid_default() {
        let cats = vec![category("cat-1"), category("cat-2")];
        let defaults = CatalogDefaults {
            main_currency_id: "cur-1".to_string(),
            default_category_id: "cat-2".to_string(),
            default_payment_method_id: "pm-1".to_string(),
        };
        assert_eq!(
            resolve_category_replacement(&cats, "cat-9", &defaults).unwrap(),
            "cat-2"
        );
    }

    #[test]
    fn resolve_category_replacement_when_default_deleted_uses_cat1() {
        let cats = vec![category("cat-1"), category("cat-2")];
        let defaults = CatalogDefaults {
            main_currency_id: "cur-1".to_string(),
            default_category_id: "cat-2".to_string(),
            default_payment_method_id: "pm-1".to_string(),
        };
        assert_eq!(
            resolve_category_replacement(&cats, "cat-2", &defaults).unwrap(),
            "cat-1"
        );
    }

    #[test]
    fn resolve_category_replacement_without_cat1_uses_other_category() {
        let cats = vec![category("cat-2"), category("cat-3")];
        let defaults = CatalogDefaults {
            main_currency_id: "cur-1".to_string(),
            default_category_id: "cat-2".to_string(),
            default_payment_method_id: "pm-1".to_string(),
        };
        assert_eq!(
            resolve_category_replacement(&cats, "cat-2", &defaults).unwrap(),
            "cat-3"
        );
    }
}
