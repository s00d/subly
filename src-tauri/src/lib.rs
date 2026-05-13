#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::Emitter;
use tauri::Manager;
use redb::{Database, ReadableDatabase, TableDefinition};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
mod commands;
pub mod errors;
pub(crate) mod keyring_store;
mod models;
mod state;
mod state_tables;
mod widget_snapshot;
#[cfg(target_os = "ios")]
mod ios_webview_fix;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod restart_smoke_tests;
pub(crate) type AppState = state::AppState;
pub(crate) use models::{AppDataDoc, RatePoint};
use commands::notifications::{
    notifications_cancel_all_scheduled,
    notifications_event,
    notifications_reschedule_all,
};
use commands::logo::{logo_get_assets, logo_resolve_favicon_from_input_url};
use commands::rates::{
    currency_get_flags,
    ensure_rates_scheduler_started,
    rates_get_providers,
    rates_run_backend_update,
    rates_should_update,
    rates_update_with_fallback,
};
use commands::seed::{seed_apply_if_empty, seed_get_default_data};
use commands::sync::{
    sync_check_remote,
    sync_disable_provider,
    sync_dismiss_pending_update,
    sync_enable_provider,
    sync_flush_before_exit,
    sync_get_settings,
    sync_get_status,
    sync_get_ui_schema,
    sync_now,
    sync_oauth_finish,
    sync_oauth_start,
    sync_pull_remote,
    sync_push_local,
    sync_force_push_local,
    sync_save_settings,
    sync_build_push_meta,
    sync_has_push_conflict,
    sync_merge_app_data,
    sync_payload_fits_limit,
    sync_should_pull,
};
use commands::subscriptions::{
    get_overdue_subscriptions,
    get_upcoming_subscriptions,
    list_subscriptions_page,
    subscriptions_next_cycle_date,
    subscriptions_payment_dates_in_month,
    subscriptions_delete,
    subscriptions_delete_batch,
    subscriptions_delete_payment_record,
    subscriptions_insert,
    subscriptions_insert_payment_record,
    subscriptions_list,
    subscriptions_upsert,
    subscriptions_update,
};
use commands::expenses::{
    expenses_for_month,
    expenses_total_filtered,
    list_expenses_page,
    expenses_count,
    expenses_delete,
    expenses_delete_batch,
    expenses_delete_by_payment_record,
    expenses_get_by_id,
    expenses_insert,
    expenses_remove_tag_batch,
    expenses_upsert,
    expenses_update,
    expenses_update_tags_batch,
};
use commands::catalog::{
    catalogs_delete_category,
    catalogs_delete_currency,
    catalogs_delete_household_member,
    catalogs_delete_payment_method,
    catalogs_delete_tag,
    catalogs_load,
    catalogs_max_sort_order,
    catalogs_usage_summary,
    catalogs_update_currency_rates,
    catalogs_upsert_category,
    catalogs_upsert_currency,
    catalogs_upsert_household_member,
    catalogs_upsert_payment_method,
    catalogs_upsert_tag,
};
use commands::dashboard::{
    get_dashboard_charts,
    get_dashboard_forecast,
    get_dashboard_summary,
    get_dashboard_trends,
};
use commands::app_data::{load_all_data, load_app_data, reset_app_data};
use commands::app_ready::app_ready;
use commands::export::{export_get_path_presets, export_subly_backup, import_subly_backup, import_subly_backup_bytes};
use commands::rates::{
    get_rate_history_widget,
    rate_history_clear,
    rate_history_count,
    rate_history_get,
    rate_history_prune,
    rate_history_save_snapshot,
};
use commands::subscription_credentials::{
    subscription_credentials_delete,
    subscription_credentials_get,
    subscription_credentials_set,
    subscription_totp_current,
    subscription_totp_decode_qr_base64,
    subscription_totp_import_otpauth,
};
use commands::storage::{redb_delete, redb_get, redb_set};
use commands::config::{config_get, config_set, config_delete};
use commands::ai::{ai_get_providers, ai_smart_input, ai_test_connection};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use commands::tray::setup_desktop_tray;

pub(crate) const KV_TABLE: TableDefinition<&str, &str> = TableDefinition::new("subly_kv");
static APP_DB: OnceLock<Arc<Database>> = OnceLock::new();

/// Resolves the absolute path to `subly.redb`.
///
/// Desktop + iOS use `dirs::data_local_dir()` which has historically been our
/// storage root: `~/Library/Application Support/Subly/` on macOS/iOS,
/// `%LOCALAPPDATA%\Subly\` on Windows, `~/.local/share/Subly/` on Linux. We
/// must keep this exact location so existing installs don't appear empty after
/// the upgrade (Tauri's `app_local_data_dir` would point at a *different*
/// directory: `…/Application Support/com.s00d.subly/…`).
///
/// Android is the special case: `dirs` returns `None` there (no XDG / no
/// `$HOME`), which used to crash the setup hook with "Unable to resolve local
/// data directory". For Android we route through Tauri's path resolver which
/// internally calls `Context.getFilesDir()` and yields the per-app sandbox
/// (`/data/user/0/<package>/files/`).
fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, crate::errors::AppError> {
    #[cfg(target_os = "android")]
    {
        let base = app
            .path()
            .app_local_data_dir()
            .map_err(|e| crate::errors::AppError::from(
                format!("Unable to resolve local data directory: {e}"),
            ))?;
        std::fs::create_dir_all(&base)?;
        return Ok(base.join("subly.redb"));
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        let base = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| {
                crate::errors::AppError::from("Unable to resolve local data directory")
            })?;
        let dir = base.join("Subly");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("subly.redb"))
    }
}

fn open_redb(app: &tauri::AppHandle) -> Result<Database, crate::errors::AppError> {
    let path = db_path(app)?;
    if path.exists() {
        Database::open(path).map_err(crate::errors::AppError::from)
    } else {
        Database::create(path).map_err(crate::errors::AppError::from)
    }
}

fn init_kv_table(db: &Database) -> Result<(), crate::errors::AppError> {
    let tx = db.begin_write()?;
    let _ = tx.open_table(KV_TABLE)?;
    tx.commit()?;
    Ok(())
}

fn open_db_with_startup_recovery(
    app: &tauri::AppHandle,
) -> Result<(Arc<Database>, AppDataDoc, models::AppConfigDoc), crate::errors::AppError> {
    let db = Arc::new(open_redb(app)?);
    init_kv_table(db.as_ref())?;
    let (data, cfg) = load_initial_state_for_startup(db.as_ref())?;
    Ok((db, data, cfg))
}

pub(crate) fn app_db() -> Result<Arc<Database>, crate::errors::AppError> {
    APP_DB
        .get()
        .cloned()
        .ok_or_else(|| crate::errors::AppError::from("database is not initialized"))
}

pub(crate) fn redb_get_internal(key: String) -> Result<Option<String>, crate::errors::AppError> {
    let db = app_db()?;
    let tx = db.begin_read()?;
    let table = tx.open_table(KV_TABLE)?;
    let maybe = table.get(key.as_str())?;
    Ok(maybe.map(|v| v.value().to_string()))
}

pub(crate) fn redb_set_internal(key: String, value: String) -> Result<(), crate::errors::AppError> {
    let db = app_db()?;
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(KV_TABLE)?;
        table.insert(key.as_str(), value.as_str())?;
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn rate_map(data: &AppDataDoc) -> std::collections::HashMap<String, f64> {
    data.currencies
        .iter()
        .map(|c| (c.id.clone(), if c.rate <= 0.0 { 1.0 } else { c.rate }))
        .collect()
}


pub(crate) fn convert_to_main(amount: f64, currency_id: &str, rates: &std::collections::HashMap<String, f64>) -> f64 {
    let rate = *rates.get(currency_id).unwrap_or(&1.0);
    if rate <= 0.0 { amount } else { amount / rate }
}

pub(crate) fn price_per_month(cycle: u8, frequency: u32, price: f64) -> f64 {
    let freq = if frequency == 0 { 1.0 } else { frequency as f64 };
    match cycle {
        // Billing every `freq` days → normalize via payments/year ÷ 12.
        1 => price * (365.25 / freq) / 12.0,
        // Every `freq` weeks (ISO-style 52 weeks/year).
        2 => price * (52.0 / freq) / 12.0,
        3 => price * (1.0 / freq),
        4 => price / (12.0 * freq),
        _ => price,
    }
}

#[cfg(test)]
mod price_tests {
    use super::price_per_month;

    #[test]
    fn weekly_once_per_week_matches_annual_over_12() {
        let monthly = price_per_month(2, 1, 12.0);
        let expected = 12.0 * (52.0_f64 / 12.0);
        assert!(
            (monthly - expected).abs() < 1e-9,
            "weekly 12/year should equal 52/12*price, got {monthly}"
        );
    }

    #[test]
    fn biweekly_normalization() {
        let monthly = price_per_month(2, 2, 30.0);
        let expected = 30.0 * (26.0_f64 / 12.0);
        assert!((monthly - expected).abs() < 1e-6);
    }
}

pub(crate) fn build_default_ui_settings(data: &AppDataDoc) -> serde_json::Value {
    let main_currency_id = data
        .currencies
        .first()
        .map(|c| c.id.clone())
        .unwrap_or_default();
    let default_category_id = data
        .categories
        .first()
        .map(|c| c.id.clone())
        .unwrap_or_default();
    let default_payment_method_id = data
        .payment_methods
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_default();
    serde_json::json!({
        "darkTheme": 2,
        "colorTheme": "blue",
        "monthlyPrice": true,
        "convertCurrency": true,
        "hideDisabled": false,
        "disabledToBottom": false,
        "showOriginalPrice": true,
        "showSubscriptionProgress": true,
        "language": "en",
        "mainCurrencyId": main_currency_id,
        "defaultCategoryId": default_category_id,
        "defaultPaymentMethodId": default_payment_method_id,
        "budget": 0.0,
        "notifyDaysBefore": 1,
        "notificationTitle": "Subly - {name}",
        "notificationBodyDueToday": "Payment for \"{name}\" is due today.",
        "notificationBodyDueSoon": "Payment for \"{name}\" is due in {days} day(s).",
        "notificationOverdueTitle": "Subly - Overdue: {name}",
        "notificationOverdueBody": "\"{name}\" is overdue by {days} day(s).",
        "notificationSchedule": "any",
        "notificationCustomHour": 9,
        "recurringNotifications": false,
        "notificationSound": false,
        "currencyAutoUpdate": false,
        "currencyUpdateTargets": if main_currency_id.is_empty() { Vec::<String>::new() } else { vec![main_currency_id.clone()] },
        "lastCurrencyUpdate": "",
        "dashboardWidgets": Vec::<serde_json::Value>::new(),
        "subscriptionViewMode": "default",
        "subscriptionGroupBy": "none",
        "expenseViewMode": "default",
        "currencyViewMode": "default",
        "calendarViewMode": "default",
        "converterPresets": vec![1, 10, 100],
        "rateHistoryEnabled": true,
        "rateHistoryDays": 90,
        "customColors": { "main": "", "accent": "", "hover": "" }
    })
}

fn is_core_snapshot_empty(data: &AppDataDoc) -> bool {
    data.subscriptions.is_empty()
        && data.expenses.is_empty()
        && data.categories.is_empty()
        && data.currencies.is_empty()
        && data.household.is_empty()
        && data.payment_methods.is_empty()
        && data.tags.is_empty()
}

fn load_initial_state_for_startup(db: &Database) -> Result<(AppDataDoc, models::AppConfigDoc), crate::errors::AppError> {
    let (mut initial_data, mut initial_config) = state::load_app_data_typed(db).map_err(|err| {
        eprintln!("[subly][fatal] typed storage read failed on startup: {}", err);
        format!("typed storage is unreadable; startup aborted to prevent data loss: {}", err)
    })?;
    if !initial_config.initialized {
        if is_core_snapshot_empty(&initial_data) {
            initial_data = seed_get_default_data()?;
        }
        initial_config.initialized = true;
        state::save_app_data_typed(db, &initial_data, &initial_config)?;
    }
    Ok((initial_data, initial_config))
}

// ---- iCloud commands (macOS / iOS only) ----

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod icloud {
    use std::path::PathBuf;

    fn container_dir() -> Option<PathBuf> {
        // Same root as sync iCloud provider (ubiquity Documents/… or macOS legacy CloudDocs/…).
        crate::commands::sync::providers::icloud::icloud_subly_sync_dir()
    }

    #[tauri::command]
    pub fn icloud_container_url() -> Option<String> {
        container_dir().map(|p| p.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn icloud_write_file(_app: tauri::AppHandle, filename: String, contents: String) -> Result<(), crate::errors::AppError> {
        let dir = container_dir().ok_or("iCloud container not available")?;
        let path = dir.join(&filename);
        crate::commands::sync::providers::icloud_native::coordinated_write_bytes(&path, contents.as_bytes())
    }

    #[tauri::command]
    pub fn icloud_read_file(_app: tauri::AppHandle, filename: String) -> Result<Option<String>, crate::errors::AppError> {
        let dir = container_dir().ok_or("iCloud container not available")?;
        let path = dir.join(&filename);
        crate::commands::sync::providers::icloud_native::coordinated_read_string(&path)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
mod icloud {
    #[tauri::command]
    pub fn icloud_container_url() -> Option<String> {
        None
    }

    #[tauri::command]
    pub fn icloud_write_file(_filename: String, _contents: String) -> Result<(), crate::errors::AppError> {
        Err("iCloud not available on this platform".into())
    }

    #[tauri::command]
    pub fn icloud_read_file(_filename: String) -> Result<Option<String>, crate::errors::AppError> {
        Err("iCloud not available on this platform".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg_attr(
        any(target_os = "android", target_os = "ios"),
        allow(unused_mut)
    )]
    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let urls: Vec<String> = args
                .iter()
                .filter(|a| a.starts_with("subly://oauth/callback"))
                .cloned()
                .collect();
            if !urls.is_empty() {
                let _ = app.emit("deep-link:single-instance", urls);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    let builder = builder
        .plugin(tauri_plugin_keyring_store::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            icloud::icloud_container_url,
            icloud::icloud_write_file,
            icloud::icloud_read_file,
            subscription_credentials_get,
            subscription_credentials_set,
            subscription_credentials_delete,
            subscription_totp_current,
            subscription_totp_import_otpauth,
            subscription_totp_decode_qr_base64,
            redb_get,
            redb_set,
            redb_delete,
            config_get,
            config_set,
            config_delete,
            load_all_data,
            load_app_data,
            reset_app_data,
            export_subly_backup,
            import_subly_backup,
            import_subly_backup_bytes,
            export_get_path_presets,
            seed_get_default_data,
            seed_apply_if_empty,
            get_dashboard_summary,
            get_dashboard_charts,
            get_dashboard_forecast,
            get_dashboard_trends,
            get_rate_history_widget,
            rates_should_update,
            rates_get_providers,
            rates_run_backend_update,
            rates_update_with_fallback,
            currency_get_flags,
            list_subscriptions_page,
            subscriptions_next_cycle_date,
            subscriptions_payment_dates_in_month,
            list_expenses_page,
            expenses_for_month,
            expenses_total_filtered,
            subscriptions_list,
            get_overdue_subscriptions,
            get_upcoming_subscriptions,
            subscriptions_insert,
            subscriptions_upsert,
            subscriptions_update,
            subscriptions_delete,
            subscriptions_delete_batch,
            subscriptions_insert_payment_record,
            subscriptions_delete_payment_record,
            expenses_get_by_id,
            expenses_insert,
            expenses_upsert,
            expenses_update,
            expenses_delete,
            expenses_delete_batch,
            expenses_delete_by_payment_record,
            expenses_count,
            expenses_update_tags_batch,
            expenses_remove_tag_batch,
            catalogs_load,
            catalogs_usage_summary,
            catalogs_upsert_category,
            catalogs_delete_category,
            catalogs_upsert_currency,
            catalogs_delete_currency,
            catalogs_update_currency_rates,
            catalogs_upsert_household_member,
            catalogs_delete_household_member,
            catalogs_upsert_payment_method,
            catalogs_delete_payment_method,
            catalogs_upsert_tag,
            catalogs_delete_tag,
            catalogs_max_sort_order,
            rate_history_save_snapshot,
            rate_history_get,
            rate_history_prune,
            rate_history_clear,
            rate_history_count,
            notifications_event,
            notifications_reschedule_all,
            notifications_cancel_all_scheduled,
            logo_get_assets,
            logo_resolve_favicon_from_input_url,
            sync_should_pull,
            sync_has_push_conflict,
            sync_merge_app_data,
            sync_build_push_meta,
            sync_payload_fits_limit,
            sync_get_ui_schema,
            sync_get_settings,
            sync_save_settings,
            sync_get_status,
            sync_enable_provider,
            sync_disable_provider,
            sync_oauth_start,
            sync_oauth_finish,
            sync_check_remote,
            sync_pull_remote,
            sync_push_local,
            sync_force_push_local,
            sync_now,
            sync_flush_before_exit,
            sync_dismiss_pending_update,
            app_ready,
            ai_get_providers,
            ai_test_connection,
            ai_smart_input,
        ]);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_autostart::Builder::new().build());

    builder
        .setup(|app| {
            let (db, initial_data, _initial_config) = open_db_with_startup_recovery(app.handle())
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            let _ = APP_DB.set(db.clone());
            if redb_get_internal("config:settings".to_string())?.is_none() {
                let settings_blob = serde_json::to_string(&build_default_ui_settings(&initial_data))
                    .map_err(|e| e.to_string())?;
                redb_set_internal("config:settings".to_string(), settings_blob)?;
            }
            app.manage(std::sync::Mutex::new(state::AppStateInner { db, app_data: initial_data }));
            ensure_rates_scheduler_started(app.handle().clone());

            #[cfg(target_os = "ios")]
            {
                widget_snapshot::export_ios_widget_snapshot_from_app(app.handle());
            }

            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.show();
                }
            }

            // iOS: stretch WKWebView from `safeAreaLayoutGuide` to the full window
            // bounds so `position: fixed; bottom: 0` actually reaches the bottom of
            // the screen (mobile tab bar). See `ios_webview_fix` for the details.
            #[cfg(target_os = "ios")]
            {
                ios_webview_fix::install(app.handle());
            }

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let app_handle = app.handle().clone();
                setup_desktop_tray(&app_handle)?;

                let dark_bg = tauri::window::Color(0x0F, 0x17, 0x2A, 0xFF);
                let light_bg = tauri::window::Color(0xF7, 0xF8, 0xFB, 0xFF);
                let is_dark = app
                    .get_webview_window("main")
                    .and_then(|w| w.theme().ok())
                    .map(|t| matches!(t, tauri::Theme::Dark))
                    .unwrap_or(true);
                let splash_bg = if is_dark { dark_bg } else { light_bg };

                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.set_background_color(Some(splash_bg));
                }

                let _ = tauri::WebviewWindowBuilder::new(
                    app,
                    "splashscreen",
                    tauri::WebviewUrl::App("splashscreen.html".into()),
                )
                .title("Subly")
                .inner_size(360.0, 420.0)
                .resizable(false)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .center()
                .background_color(splash_bg)
                .build();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                let _ = (window, event);
            }
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{save_app_data_typed, T2_CONFIG};

    #[test]
    fn startup_loads_seed_when_config_not_initialized() {
        let (_dir, db) = crate::test_support::temp_db().expect("create temp db");
        let doc = crate::test_support::base_seeded_doc().expect("seed");
        let mut cfg = models::AppConfigDoc::default();
        cfg.initialized = false;
        save_app_data_typed(&db, &doc, &cfg).expect("save");

        let (loaded, loaded_cfg) = load_initial_state_for_startup(&db).expect("startup load");
        assert!(loaded_cfg.initialized, "startup should force initialized=true");
        assert!(!loaded.categories.is_empty(), "seeded categories should exist");
    }

    #[test]
    fn startup_does_not_seed_over_non_empty_snapshot_when_config_not_initialized() {
        let (_dir, db) = crate::test_support::temp_db().expect("create temp db");
        let mut doc = crate::test_support::base_seeded_doc().expect("seed");
        doc.currencies = vec![crate::models::CurrencyDoc {
            id: "cur-custom-1".to_string(),
            name: "Custom".to_string(),
            symbol: "C".to_string(),
            code: "CUS".to_string(),
            rate: 1.0,
            sort_order: 0,
            i18n_key: String::new(),
        }];
        let mut cfg = models::AppConfigDoc::default();
        cfg.initialized = false;
        save_app_data_typed(&db, &doc, &cfg).expect("save");

        let (loaded, loaded_cfg) = load_initial_state_for_startup(&db).expect("startup load");
        assert!(loaded_cfg.initialized, "startup should force initialized=true");
        assert_eq!(loaded.currencies.len(), 1, "currencies should be preserved");
        assert_eq!(loaded.currencies[0].id, "cur-custom-1");
    }

    #[test]
    fn startup_fails_fast_on_corrupted_typed_storage() {
        let (_dir, db) = crate::test_support::temp_db().expect("create temp db");
        let tx = db.begin_write().expect("begin write");
        {
            let mut table = tx.open_table(T2_CONFIG).expect("open config table");
            table.insert("data", &[1_u8, 2_u8, 3_u8][..]).expect("insert corrupted payload");
        }
        tx.commit().expect("commit");

        let err = load_initial_state_for_startup(&db).expect_err("startup should fail");
        assert!(
            err.to_string().contains("typed storage is unreadable"),
            "error should describe strict fail-fast startup"
        );
    }

    #[test]
    fn default_ui_settings_contains_main_currency_id() {
        let doc = crate::test_support::base_seeded_doc().expect("seed");
        let settings = build_default_ui_settings(&doc);
        let main = settings
            .get("mainCurrencyId")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert!(!main.is_empty(), "default settings should include non-empty mainCurrencyId");
    }
}
