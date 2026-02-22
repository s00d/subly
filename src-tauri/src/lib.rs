use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};

fn migration(version: i64, description: &'static str, sql: &'static str) -> Migration {
    Migration { version, description, sql, kind: MigrationKind::Up }
}

fn get_migrations() -> Vec<Migration> {
    vec![
        migration(1, "create_initial_tables", include_str!("migrations/001_create_initial_tables.sql")),
        migration(2, "add_rate_history", include_str!("migrations/002_add_rate_history.sql")),
    ]
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::tray::TrayIconBuilder;

// ---- iCloud commands (macOS / iOS only) ----

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod icloud {
    use std::path::PathBuf;

    fn container_dir() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        // Use the general iCloud Drive folder (com~apple~CloudDocs) which is
        // available to all users with iCloud Drive enabled, no entitlements needed.
        let cloud_docs = home.join("Library/Mobile Documents/com~apple~CloudDocs");
        if !cloud_docs.exists() {
            return None;
        }
        let subly_dir = cloud_docs.join("Subly");
        if subly_dir.exists() || std::fs::create_dir_all(&subly_dir).is_ok() {
            Some(subly_dir)
        } else {
            None
        }
    }

    #[tauri::command]
    pub fn icloud_container_url() -> Option<String> {
        container_dir().map(|p| p.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn icloud_write_file(filename: String, contents: String) -> Result<(), String> {
        let dir = container_dir().ok_or("iCloud container not available")?;
        let path = dir.join(&filename);
        std::fs::write(&path, &contents).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn icloud_read_file(filename: String) -> Result<Option<String>, String> {
        let dir = container_dir().ok_or("iCloud container not available")?;
        let path = dir.join(&filename);
        if !path.exists() {
            return Ok(None);
        }
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| e.to_string())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
mod icloud {
    #[tauri::command]
    pub fn icloud_container_url() -> Option<String> {
        None
    }

    #[tauri::command]
    pub fn icloud_write_file(_filename: String, _contents: String) -> Result<(), String> {
        Err("iCloud not available on this platform".into())
    }

    #[tauri::command]
    pub fn icloud_read_file(_filename: String) -> Result<Option<String>, String> {
        Err("iCloud not available on this platform".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_notifications::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:subly.db", get_migrations())
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            icloud::icloud_container_url,
            icloud::icloud_write_file,
            icloud::icloud_read_file,
        ]);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_autostart::Builder::new().build());

    builder
        .setup(|app| {
            // Request notification permission by sending a silent init notification
            // This triggers the OS permission dialog on first launch (macOS/iOS)
            let _ = app
                .notification()
                .builder()
                .title("Subly")
                .body("Notifications enabled")
                .show();

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                // Create initial tray menu (desktop only)
                let show_i = MenuItem::with_id(app, "show", "Open Subly", true, None::<&str>)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &separator, &quit_i])?;

                let icon = app.default_window_icon().cloned().expect("no default icon");
                let _tray = TrayIconBuilder::with_id("main-tray")
                    .icon(icon)
                    .tooltip("Subly")
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
