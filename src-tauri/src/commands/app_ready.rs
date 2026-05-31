use tauri::Manager;
use crate::errors::AppError;

#[tauri::command]
pub async fn app_ready(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }

    // After DB + UI are up, refresh home-screen widget data (setup may run before the webview is ready).
    #[cfg(any(target_os = "ios", target_os = "android"))]
    crate::widget_snapshot::export_widget_snapshot_from_app(&app);

    Ok(())
}
