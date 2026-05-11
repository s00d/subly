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
    Ok(())
}
