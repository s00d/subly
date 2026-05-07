#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::menu::{Menu, MenuItem};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::tray::TrayIconBuilder;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::Manager;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn refresh_tray_menu(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<crate::AppState>();
    let data = {
        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
        guard.doc()?
    };
    let rates = crate::rate_map(&data);
    let today = chrono::Local::now().date_naive();
    let end = today + chrono::Duration::days(7);

    let main_currency = data
        .currencies
        .iter()
        .find(|c| c.id == data.settings.main_currency_id)
        .or_else(|| data.currencies.first());
    let main_currency_code = main_currency.map(|c| c.code.as_str()).unwrap_or("USD");

    let active_count = data.subscriptions.iter().filter(|s| !s.inactive).count();
    let monthly_cost = data
        .subscriptions
        .iter()
        .filter(|s| !s.inactive)
        .map(|s| {
            let in_main = crate::convert_to_main(s.price, &s.currency_id, &rates);
            crate::price_per_month(s.cycle, s.frequency, in_main)
        })
        .sum::<f64>();

    let mut overdue = data
        .subscriptions
        .iter()
        .filter(|s| {
            if s.inactive {
                return false;
            }
            chrono::NaiveDate::parse_from_str(&s.next_payment, "%Y-%m-%d")
                .map(|d| d < today)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    overdue.sort_by(|a, b| a.next_payment.cmp(&b.next_payment));
    overdue.truncate(5);

    let mut upcoming = data
        .subscriptions
        .iter()
        .filter(|s| {
            if s.inactive {
                return false;
            }
            chrono::NaiveDate::parse_from_str(&s.next_payment, "%Y-%m-%d")
                .map(|d| d >= today && d <= end)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    upcoming.sort_by(|a, b| a.next_payment.cmp(&b.next_payment));
    upcoming.truncate(8);

    let fmt = |amount: f64, currency_code: &str| -> String {
        let mut value = amount;
        if !value.is_finite() {
            value = 0.0;
        }
        format!("{value:.2} {currency_code}")
    };

    let mut items: Vec<tauri::menu::MenuItem<_>> = Vec::new();
    items.push(MenuItem::with_id(app, "tray-header", format!("Monthly: {}", fmt(monthly_cost, main_currency_code)), false, None::<&str>).map_err(|e| e.to_string())?);
    items.push(MenuItem::with_id(app, "tray-sep-1", "----------------", false, None::<&str>).map_err(|e| e.to_string())?);

    if !overdue.is_empty() {
        items.push(MenuItem::with_id(app, "tray-overdue-header", format!("Overdue ({})", overdue.len()), false, None::<&str>).map_err(|e| e.to_string())?);
        for s in overdue {
            let cur_code = data
                .currencies
                .iter()
                .find(|c| c.id == s.currency_id)
                .map(|c| c.code.as_str())
                .unwrap_or("USD");
            let text = format!("! {} - {}", s.name, fmt(s.price, cur_code));
            items.push(MenuItem::with_id(app, format!("tray-open-sub-{}", s.id), text, true, None::<&str>).map_err(|e| e.to_string())?);
        }
        items.push(MenuItem::with_id(app, "tray-sep-2", "----------------", false, None::<&str>).map_err(|e| e.to_string())?);
    }

    if !upcoming.is_empty() {
        items.push(MenuItem::with_id(app, "tray-upcoming-header", "Upcoming (7 days)", false, None::<&str>).map_err(|e| e.to_string())?);
        for s in upcoming {
            let cur_code = data
                .currencies
                .iter()
                .find(|c| c.id == s.currency_id)
                .map(|c| c.code.as_str())
                .unwrap_or("USD");
            let text = format!("{} - {} ({})", s.name, fmt(s.price, cur_code), s.next_payment);
            items.push(MenuItem::with_id(app, format!("tray-open-sub-{}", s.id), text, true, None::<&str>).map_err(|e| e.to_string())?);
        }
    } else {
        items.push(MenuItem::with_id(app, "tray-no-upcoming", "No upcoming payments", false, None::<&str>).map_err(|e| e.to_string())?);
    }

    items.push(MenuItem::with_id(app, "tray-sep-3", "----------------", false, None::<&str>).map_err(|e| e.to_string())?);
    items.push(MenuItem::with_id(app, "tray-active-count", format!("Active: {active_count}"), false, None::<&str>).map_err(|e| e.to_string())?);
    items.push(MenuItem::with_id(app, "tray-sep-4", "----------------", false, None::<&str>).map_err(|e| e.to_string())?);

    items.push(MenuItem::with_id(app, "show", "Open Subly", true, None::<&str>).map_err(|e| e.to_string())?);
    items.push(MenuItem::with_id(app, "tray-sep-5", "----------------", false, None::<&str>).map_err(|e| e.to_string())?);
    items.push(MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?);

    let item_refs = items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect::<Vec<_>>();
    let menu = Menu::with_items(app, &item_refs).map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        tray.set_tooltip(Some(&format!("Subly - {}/mo", fmt(monthly_cost, main_currency_code))))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn tray_refresh_signature() -> Result<String, String> {
    let today = chrono::Local::now().date_naive().to_string();
    let db = crate::open_redb()?;
    let (data, cfg) = crate::state::load_app_data_typed(&db)?;
    let payload = serde_json::to_string(&(data, cfg)).map_err(|e| e.to_string())?;
    Ok(format!("{today}|{payload}"))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn setup_desktop_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    // Bundle icon comes from tauri.conf / app icons; dev and release always set it.
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default window icon missing: check tauri bundle icons and build config");
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Subly")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                show_main_window(app);
            }
            id if id.starts_with("tray-open-sub-") => {
                show_main_window(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                let app = tray.app_handle();
                show_main_window(&app);
            }
        })
        .build(app)?;

    let app_handle = app.clone();
    let _ = refresh_tray_menu(&app_handle);
    std::thread::spawn(move || {
        let mut last_signature = tray_refresh_signature().ok();
        loop {
            match tray_refresh_signature() {
                Ok(next_signature) => {
                    let changed = last_signature
                        .as_ref()
                        .map(|prev| prev != &next_signature)
                        .unwrap_or(true);
                    if changed {
                        let _ = refresh_tray_menu(&app_handle);
                        last_signature = Some(next_signature);
                    }
                }
                Err(_) => {
                    let _ = refresh_tray_menu(&app_handle);
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    });

    Ok(())
}
