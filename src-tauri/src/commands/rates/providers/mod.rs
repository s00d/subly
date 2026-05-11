use tauri::{Manager, State};
use crate::AppState;
use crate::state::EntityTable;
use std::sync::OnceLock;
mod frankfurter;
mod fixer;
mod apilayer;
mod exchangerate;
mod currencyapi;
mod openexchangerates;

pub struct ProviderDescriptor {
    pub provider_type: &'static str,
    pub name: &'static str,
    pub requires_key: bool,
    pub free_tier_note: &'static str,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatesProviderMetaDto {
    pub r#type: String,
    pub name: String,
    pub requires_key: bool,
    pub free_tier_note: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RatesUpdateOptionsDto {
    #[serde(default)]
    pub history_enabled: bool,
    #[serde(default = "default_history_days")]
    pub history_days: i64,
}

fn default_history_days() -> i64 { 90 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatesUpdateResultDto {
    pub updated: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CurrencyRowDto {
    id: String,
    code: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RatesSettingsDto {
    #[serde(default)]
    currency_auto_update: bool,
    #[serde(default)]
    last_currency_update: String,
    #[serde(default)]
    main_currency_id: String,
    #[serde(default)]
    currency_update_targets: Vec<String>,
    #[serde(default)]
    rate_history_enabled: bool,
    #[serde(default = "default_history_days")]
    rate_history_days: i64,
}

impl Default for RatesSettingsDto {
    fn default() -> Self {
        Self {
            currency_auto_update: false,
            last_currency_update: String::new(),
            main_currency_id: String::new(),
            currency_update_targets: Vec::new(),
            rate_history_enabled: false,
            rate_history_days: default_history_days(),
        }
    }
}

const CONFIG_SETTINGS_KEY: &str = "config:settings";
const CONFIG_PROVIDER_KEY: &str = "config:ratesProvider";
const SECURE_RATES_API_KEY: &str = "secure_storage.ratesApiKey";

fn daily_update_allowed(last_update: &str) -> Result<bool, crate::errors::AppError> {
    if last_update.trim().is_empty() {
        return Ok(true);
    }
    let parsed = chrono::NaiveDate::parse_from_str(last_update.trim(), "%Y-%m-%d")
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let now = chrono::Local::now().date_naive();
    Ok((now - parsed).num_days() >= 1)
}

fn today_ymd() -> String {
    chrono::Local::now().date_naive().format("%Y-%m-%d").to_string()
}

fn load_rates_settings() -> Result<RatesSettingsDto, crate::errors::AppError> {
    let raw = crate::redb_get_internal(CONFIG_SETTINGS_KEY.to_string())?;
    match raw {
        Some(value) => serde_json::from_str::<RatesSettingsDto>(&value).map_err(|e| crate::errors::AppError::Message(e.to_string())),
        None => Ok(RatesSettingsDto::default()),
    }
}

fn update_last_currency_update(next_date: &str) -> Result<(), crate::errors::AppError> {
    let mut settings_value = match crate::redb_get_internal(CONFIG_SETTINGS_KEY.to_string())? {
        Some(raw) => serde_json::from_str::<serde_json::Value>(&raw).map_err(|e| crate::errors::AppError::Message(e.to_string()))?,
        None => serde_json::json!({}),
    };
    if !settings_value.is_object() {
        settings_value = serde_json::json!({});
    }
    settings_value["lastCurrencyUpdate"] = serde_json::Value::String(next_date.to_string());
    crate::redb_set_internal(
        CONFIG_SETTINGS_KEY.to_string(),
        serde_json::to_string(&settings_value).map_err(|e| crate::errors::AppError::Message(e.to_string()))?,
    )
}

fn load_provider_and_key() -> Result<(String, String), crate::errors::AppError> {
    let provider = crate::redb_get_internal(CONFIG_PROVIDER_KEY.to_string())?
        .and_then(|raw| serde_json::from_str::<String>(&raw).ok())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "frankfurter".to_string());
    let api_key = crate::keyring_store::get(SECURE_RATES_API_KEY)?.unwrap_or_default();
    eprintln!(
        "[subly][rates] config provider={} api_key_present={} api_key_len={}",
        provider,
        !api_key.trim().is_empty(),
        api_key.trim().len()
    );
    Ok((provider, api_key))
}

static RATES_UPDATE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn resolve_main_currency<'a>(currencies: &'a [CurrencyRowDto], main_currency_id: &str) -> Result<&'a CurrencyRowDto, crate::errors::AppError> {
    if currencies.is_empty() {
        return Err(crate::errors::AppError::from("No currencies available for update"));
    }
    currencies
        .iter()
        .find(|c| c.id == main_currency_id)
        .or_else(|| currencies.iter().find(|c| c.code.eq_ignore_ascii_case(main_currency_id)))
        .or_else(|| currencies.first())
        .ok_or_else(|| crate::errors::AppError::from("Main currency not found"))
}

fn currency_rows_from_docs(currencies: &[crate::models::CurrencyDoc]) -> Vec<CurrencyRowDto> {
    currencies
        .iter()
        .map(|c| CurrencyRowDto {
            id: c.id.clone(),
            code: c.code.clone(),
        })
        .collect()
}

fn resolve_target_ids(
    currencies: &[CurrencyRowDto],
    raw_targets: Vec<String>,
    main_id: &str,
) -> Vec<String> {
    if raw_targets.is_empty() {
        return currencies
            .iter()
            .map(|c| c.id.clone())
            .filter(|id| id != main_id)
            .collect();
    }

    let mut out: Vec<String> = Vec::new();
    for raw in raw_targets {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }
        let resolved = currencies
            .iter()
            .find(|c| c.id == token)
            .or_else(|| currencies.iter().find(|c| c.code.eq_ignore_ascii_case(token)))
            .map(|c| c.id.clone());
        if let Some(id) = resolved {
            if id != main_id && !out.contains(&id) {
                out.push(id);
            }
        }
    }
    if out.is_empty() {
        currencies
            .iter()
            .map(|c| c.id.clone())
            .filter(|id| id != main_id)
            .collect()
    } else {
        out
    }
}

fn normalize_currency_code(raw: &str) -> String {
    raw.trim().to_ascii_uppercase()
}

fn normalize_rates_map(
    rates: std::collections::HashMap<String, f64>,
) -> std::collections::HashMap<String, f64> {
    let mut out = std::collections::HashMap::new();
    for (code, rate) in rates {
        let normalized = normalize_currency_code(&code);
        if !normalized.is_empty() {
            out.insert(normalized, rate);
        }
    }
    out
}

fn has_target_coverage(
    rates: &std::collections::HashMap<String, f64>,
    target_codes: &[String],
) -> bool {
    target_codes.iter().any(|code| rates.contains_key(code))
}

async fn perform_rates_update(
    state: &AppState,
    provider_type: String,
    api_key: String,
    main_currency_id: String,
    target_currency_ids: Vec<String>,
    opts: Option<RatesUpdateOptionsDto>,
) -> Result<RatesUpdateResultDto, crate::errors::AppError> {
    eprintln!(
        "[subly][rates] perform update provider={} main_currency_id={} targets={:?}",
        provider_type, main_currency_id, target_currency_ids
    );
    let currencies_docs = {
        let guard = state
            .lock()
            .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        guard.table_list_typed::<crate::models::CurrencyDoc>(EntityTable::Currencies)?
    };
    let currencies = currency_rows_from_docs(&currencies_docs);
    let main = resolve_main_currency(&currencies, &main_currency_id)?;
    let main_id = main.id.clone();
    let main_code = normalize_currency_code(&main.code);
    if main_code.is_empty() {
        return Err(crate::errors::AppError::from("Main currency code missing"));
    }
    let target_ids = resolve_target_ids(&currencies, target_currency_ids, &main_id);

    if target_ids.is_empty() {
        return Err(crate::errors::AppError::from("No target currencies configured for rates update"));
    }

    let target_codes: Vec<String> = target_ids
        .iter()
        .filter_map(|id| {
            currencies
                .iter()
                .find(|c| c.id == *id)
                .map(|c| normalize_currency_code(&c.code))
        })
        .filter(|code| !code.is_empty())
        .collect();
    if target_codes.is_empty() {
        return Err(crate::errors::AppError::from("No valid target currency codes configured"));
    }

    let provider_chain: Vec<&str> = {
        let mut chain = vec![provider_type.as_str(), "frankfurter"];
        chain.dedup();
        chain
    };

    let mut fetched: Option<std::collections::HashMap<String, f64>> = None;
    let mut last_error = String::new();
    let mut provider_errors: Vec<String> = Vec::new();
    for candidate in provider_chain {
        eprintln!(
            "[subly][rates] try provider={} main_code={} target_codes={:?}",
            candidate, main_code, target_codes
        );
        let fetched_for_provider = match candidate {
            "frankfurter" => frankfurter::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            "fixer" => fixer::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            "apilayer" => apilayer::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            "exchangerate" => exchangerate::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            "currencyapi" => currencyapi::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            "openexchangerates" => openexchangerates::fetch_rates(&main_code, &target_codes, api_key.as_str()).await,
            _ => Err(crate::errors::AppError::from("provider not implemented")),
        };
        match fetched_for_provider {
            Ok(rates) => {
                let normalized = normalize_rates_map(rates);
                eprintln!(
                    "[subly][rates] provider={} fetched_count={}",
                    candidate,
                    normalized.len()
                );
                if normalized.is_empty() {
                    last_error = format!("{} returned empty rates payload", candidate);
                    provider_errors.push(last_error.clone());
                    continue;
                }
                if !has_target_coverage(&normalized, &target_codes) {
                    last_error = format!("{} returned no rates for configured targets", candidate);
                    provider_errors.push(last_error.clone());
                    continue;
                }
                fetched = Some(normalized);
                break;
            }
            Err(e) => {
                eprintln!("[subly][rates] provider={} error={}", candidate, e);
                last_error = format!("{}: {}", candidate, e);
                provider_errors.push(last_error.clone());
            }
        }
    }
    let rates = fetched.ok_or_else(|| {
        if !provider_errors.is_empty() {
            format!("Rates update failed: {}", provider_errors.join(" | "))
        } else if last_error.is_empty() {
            "Rates update failed: unknown provider error".to_string()
        } else {
            format!("Rates update failed: {}", last_error)
        }
    })?;

    let opts = opts.unwrap_or_default();
    let history_enabled = opts.history_enabled;
    let history_days = opts.history_days;

    let mut rate_by_id = std::collections::HashMap::new();
    for id in &target_ids {
        if let Some(code) = currencies
            .iter()
            .find(|c| c.id == *id)
            .map(|c| normalize_currency_code(&c.code))
        {
            if !code.is_empty() {
                if let Some(rate) = rates.get(&code) {
                    rate_by_id.insert(id.clone(), *rate);
                }
            }
        }
    }
    if rate_by_id.is_empty() {
        return Err(crate::errors::AppError::from(format!(
            "Provider returned no rates for target currencies: {}",
            target_codes.join(", ")
        )));
    }

    let updated = {
        let mut guard = state
            .lock()
            .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        let mut currencies = guard.table_list_typed::<crate::models::CurrencyDoc>(EntityTable::Currencies)?;
        let mut updated = 0usize;
        let mut touched_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (id, rate) in &rate_by_id {
            if let Some(cur) = currencies.iter_mut().find(|c| c.id == *id) {
                if !cur.rate.is_finite() || (cur.rate - *rate).abs() > f64::EPSILON {
                    cur.rate = *rate;
                    updated += 1;
                    let _ = history_enabled;
                    touched_ids.insert(cur.id.clone());
                }
            }
        }
        if let Some(main) = currencies.iter_mut().find(|c| c.id == main_id) {
            if !main.rate.is_finite() || (main.rate - 1.0).abs() > f64::EPSILON {
                main.rate = 1.0;
                let _ = history_days;
                touched_ids.insert(main.id.clone());
            }
        }
        for cur in &currencies {
            if touched_ids.contains(&cur.id) {
                guard.table_upsert_typed(EntityTable::Currencies, cur, &cur.id)?;
            }
        }
        updated
    };
    if updated == 0 {
        return Err(crate::errors::AppError::from("No rates changed for selected currencies"));
    }
    Ok(RatesUpdateResultDto { updated, error: None })
}

async fn run_currency_update_if_needed(
    state: &AppState,
    manual_trigger: bool,
) -> Result<RatesUpdateResultDto, crate::errors::AppError> {
    eprintln!(
        "[subly][rates] run_currency_update_if_needed start manual_trigger={}",
        manual_trigger
    );
    let _guard = RATES_UPDATE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;

    let settings = load_rates_settings()?;
    eprintln!(
        "[subly][rates] settings auto_update={} last_update={} main_currency_id={} targets_count={}",
        settings.currency_auto_update,
        settings.last_currency_update,
        settings.main_currency_id,
        settings.currency_update_targets.len()
    );
    if !manual_trigger && !settings.currency_auto_update {
        eprintln!("[subly][rates] skip: auto update disabled and not manual trigger");
        return Ok(RatesUpdateResultDto { updated: 0, error: None });
    }
    if !manual_trigger && !daily_update_allowed(&settings.last_currency_update)? {
        eprintln!("[subly][rates] skip: daily update window not reached");
        return Ok(RatesUpdateResultDto { updated: 0, error: None });
    }
    if settings.main_currency_id.trim().is_empty() {
        eprintln!("[subly][rates] fail: main currency is not configured");
        return Ok(RatesUpdateResultDto {
            updated: 0,
            error: Some("Main currency is not configured".to_string()),
        });
    }

    let (provider_type, api_key) = load_provider_and_key()?;
    let provider_requires_key = provider_descriptors()
        .into_iter()
        .find(|p| p.provider_type == provider_type)
        .map(|p| p.requires_key)
        .unwrap_or(false);
    if provider_requires_key && api_key.trim().is_empty() {
        eprintln!(
            "[subly][rates] fail: provider={} requires api key but key is empty",
            provider_type
        );
        return Ok(RatesUpdateResultDto {
            updated: 0,
            error: Some("API key required for selected rates provider".to_string()),
        });
    }

    let result = perform_rates_update(
        state,
        provider_type,
        api_key,
        settings.main_currency_id,
        settings.currency_update_targets,
        Some(RatesUpdateOptionsDto {
            history_enabled: settings.rate_history_enabled,
            history_days: settings.rate_history_days,
        }),
    )
    .await?;

    if result.updated > 0 {
        update_last_currency_update(&today_ymd())?;
    }
    eprintln!(
        "[subly][rates] run_currency_update_if_needed done updated={} error={:?}",
        result.updated, result.error
    );

    Ok(result)
}

fn provider_descriptors() -> Vec<ProviderDescriptor> {
    vec![
        frankfurter::descriptor(),
        fixer::descriptor(),
        apilayer::descriptor(),
        exchangerate::descriptor(),
        currencyapi::descriptor(),
        openexchangerates::descriptor(),
    ]
}

#[tauri::command]
pub fn rates_should_update(last_update: String) -> Result<bool, crate::errors::AppError> {
    daily_update_allowed(&last_update)
}

#[tauri::command]
pub fn rates_get_providers() -> Result<Vec<RatesProviderMetaDto>, crate::errors::AppError> {
    Ok(provider_descriptors()
        .into_iter()
        .map(|d| RatesProviderMetaDto {
            r#type: d.provider_type.to_string(),
            name: d.name.to_string(),
            requires_key: d.requires_key,
            free_tier_note: d.free_tier_note.to_string(),
        })
        .collect())
}

#[tauri::command]
pub async fn rates_update_with_fallback(
    state: State<'_, AppState>,
    provider_type: String,
    _api_key: String,
    main_currency_id: String,
    target_currency_ids: Vec<String>,
    opts: Option<RatesUpdateOptionsDto>,
) -> Result<RatesUpdateResultDto, crate::errors::AppError> {
    perform_rates_update(
        &state,
        provider_type,
        _api_key,
        main_currency_id,
        target_currency_ids,
        opts,
    ).await
}

#[tauri::command]
pub async fn rates_run_backend_update(
    state: State<'_, AppState>,
) -> Result<RatesUpdateResultDto, crate::errors::AppError> {
    eprintln!("[subly][rates] rates_run_backend_update invoked");
    let result = run_currency_update_if_needed(&state, true).await;
    match &result {
        Ok(v) => eprintln!(
            "[subly][rates] rates_run_backend_update result updated={} error={:?}",
            v.updated, v.error
        ),
        Err(e) => eprintln!("[subly][rates] rates_run_backend_update error={}", e),
    }
    result
}

pub fn ensure_rates_scheduler_started(app: tauri::AppHandle) {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        tauri::async_runtime::spawn(async move {
            loop {
                let state = app.state::<AppState>();
                match run_currency_update_if_needed(&state, false).await {
                    Ok(result) => {
                        if let Some(err) = result.error {
                            eprintln!("[subly][rates][scheduler] update completed with error: {}", err);
                        } else if result.updated == 0 {
                            eprintln!("[subly][rates][scheduler] update completed with zero changes");
                        }
                    }
                    Err(err) => {
                        eprintln!("[subly][rates][scheduler] update failed: {}", err);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::{
        has_target_coverage, normalize_currency_code, normalize_rates_map, resolve_main_currency,
        resolve_target_ids, CurrencyRowDto,
    };

    #[test]
    fn resolve_main_currency_falls_back_to_code_and_first_item() {
        let currencies = vec![
            CurrencyRowDto { id: "cur-1".to_string(), code: "EUR".to_string() },
            CurrencyRowDto { id: "cur-2".to_string(), code: "USD".to_string() },
        ];

        let by_code = resolve_main_currency(&currencies, "usd").expect("resolve by code");
        assert_eq!(by_code.id, "cur-2");

        let fallback_first = resolve_main_currency(&currencies, "missing").expect("fallback to first");
        assert_eq!(fallback_first.id, "cur-1");
    }

    #[test]
    fn resolve_main_currency_errors_when_empty() {
        let currencies: Vec<CurrencyRowDto> = Vec::new();
        let err = resolve_main_currency(&currencies, "cur-1").expect_err("should fail");
        assert!(
            err.to_string().contains("No currencies available"),
            "should explain empty currency list"
        );
    }

    #[test]
    fn resolve_target_ids_accepts_both_ids_and_codes() {
        let currencies = vec![
            CurrencyRowDto { id: "cur-1".to_string(), code: "EUR".to_string() },
            CurrencyRowDto { id: "cur-2".to_string(), code: "USD".to_string() },
            CurrencyRowDto { id: "cur-3".to_string(), code: "RUB".to_string() },
        ];
        let target_ids = resolve_target_ids(
            &currencies,
            vec!["cur-2".to_string(), "rub".to_string(), "UNKNOWN".to_string()],
            "cur-1",
        );
        assert_eq!(target_ids, vec!["cur-2".to_string(), "cur-3".to_string()]);
    }

    #[test]
    fn resolve_target_ids_falls_back_to_all_when_config_is_stale() {
        let currencies = vec![
            CurrencyRowDto { id: "cur-1".to_string(), code: "EUR".to_string() },
            CurrencyRowDto { id: "cur-2".to_string(), code: "USD".to_string() },
            CurrencyRowDto { id: "cur-3".to_string(), code: "RUB".to_string() },
        ];
        let target_ids = resolve_target_ids(
            &currencies,
            vec!["LEGACY_BAD_VALUE".to_string()],
            "cur-1",
        );
        assert_eq!(target_ids, vec!["cur-2".to_string(), "cur-3".to_string()]);
    }

    #[test]
    fn normalize_currency_code_trims_and_uppercases() {
        assert_eq!(normalize_currency_code(" usd "), "USD");
        assert_eq!(normalize_currency_code(""), "");
    }

    #[test]
    fn provider_coverage_detects_missing_targets() {
        let rates = normalize_rates_map(
            vec![("usd".to_string(), 1.0), ("eur".to_string(), 0.9)]
                .into_iter()
                .collect(),
        );
        assert!(has_target_coverage(&rates, &["USD".to_string()]));
        assert!(!has_target_coverage(&rates, &["RUB".to_string()]));
    }
}
