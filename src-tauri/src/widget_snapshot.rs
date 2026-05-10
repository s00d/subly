//! Writes a JSON snapshot for the iOS Home Screen widget (App Group shared container).
//! The widget extension cannot access Tauri/reddb; it only reads `widget/upcoming.json`.

#[cfg(target_os = "ios")]
mod ios_impl {
    use crate::commands::subscriptions::collect_upcoming_subscription_docs_for_widget;
    use crate::models::CurrencyDoc;
    use crate::state::EntityTable;
    use crate::state::AppStateInner;
    use base64::Engine;
    use objc2::rc::autoreleasepool;
    use objc2_foundation::{NSFileManager, NSString};
    use serde::Serialize;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub(super) const APP_GROUP_ID: &str = "group.com.s00d.subly";
    const RELATIVE_WIDGET_DIR: &str = "widget";
    const ICON_SUBDIR: &str = "icons";
    /// Raster PNGs keyed by SHA256(trimmed URL); kept until URL changes or user clears app data.
    const HTTP_LOGO_CACHE_SUBDIR: &str = "logo_cache";
    const UPCOMING_FILE: &str = "upcoming.json";

    #[derive(Serialize)]
    struct WidgetUpcomingItem {
        id: String,
        name: String,
        #[serde(rename = "nextPayment")]
        next_payment: String,
        price: f64,
        #[serde(rename = "currencyId")]
        currency_id: String,
        /// ISO 4217 for locale-aware currency formatting (matches web Intl / fmtCurrency).
        #[serde(rename = "currencyCode")]
        currency_code: String,
        /// Relative to `widget/`: raster PNG written by the app for the widget (`icons/<id>.png`).
        #[serde(rename = "iconFile", skip_serializing_if = "Option::is_none")]
        icon_file: Option<String>,
    }

    #[derive(Serialize)]
    struct WidgetSnapshot {
        #[serde(rename = "updatedAt")]
        updated_at: String,
        items: Vec<WidgetUpcomingItem>,
    }

    /// `widget_reload.swift` defines this with `@_cdecl`; it is linked into the app binary, not into
    /// `libapp.a`. A direct `extern "C"` would leave an unresolved symbol when `cargo` links the Rust
    /// static library — resolve at runtime instead.
    fn reload_widget_timelines_if_swift_linked() {
        use std::ffi::CString;
        for name in ["_subly_reload_widget_timelines", "subly_reload_widget_timelines"] {
            let Ok(cstr) = CString::new(name) else {
                continue;
            };
            unsafe {
                let ptr = libc::dlsym(libc::RTLD_DEFAULT, cstr.as_ptr());
                if ptr.is_null() {
                    continue;
                }
                let f: extern "C" fn() = std::mem::transmute(ptr);
                f();
                return;
            }
        }
    }

    fn app_group_root() -> Option<PathBuf> {
        autoreleasepool(|_pool| {
            let fm = NSFileManager::defaultManager();
            let url = fm.containerURLForSecurityApplicationGroupIdentifier(&NSString::from_str(APP_GROUP_ID))?;
            let path = url.path()?;
            Some(PathBuf::from(path.to_string()))
        })
    }

    fn data_uri_header_is_svg(data_uri: &str) -> bool {
        let head = data_uri.get(..160).unwrap_or(data_uri);
        head.contains("image/svg+xml") || head.contains("svg+xml")
    }

    fn decode_data_uri_bytes(data_uri: &str) -> Option<Vec<u8>> {
        let payload = data_uri.split(',').nth(1)?;
        base64::engine::general_purpose::STANDARD
            .decode(payload.trim())
            .ok()
    }

    fn sha256_hex_url_key(url: &str) -> String {
        let mut h = Sha256::new();
        h.update(url.trim().as_bytes());
        format!("{:x}", h.finalize())
    }

    fn read_http_logo_cache_hit(cache_dir: &Path, url: &str) -> Option<Vec<u8>> {
        let key = sha256_hex_url_key(url);
        let path = cache_dir.join(format!("{}.png", key));
        if !path.is_file() {
            return None;
        }
        let bytes = fs::read(path).ok()?;
        if bytes.is_empty() {
            return None;
        }
        Some(bytes)
    }

    fn write_http_logo_cache(cache_dir: &Path, url: &str, png: &[u8]) {
        let key = sha256_hex_url_key(url);
        let path = cache_dir.join(format!("{}.png", key));
        let _ = fs::write(path, png);
    }

    fn fetch_http_logo_bytes(url: &str) -> Option<Vec<u8>> {
        let mut resp = ureq::get(url).call().ok()?;
        let buf = resp.body_mut().read_to_vec().ok()?;
        if buf.len() > 512 * 1024 {
            return None;
        }
        Some(buf)
    }

    /// Uses on-disk PNG cache for HTTP(S) logos; network only on first successful fetch per URL.
    fn http_logo_raster_cached(cache_dir: &Path, url: &str) -> Option<Vec<u8>> {
        if let Some(cached) = read_http_logo_cache_hit(cache_dir, url) {
            return Some(cached);
        }
        let raw = fetch_http_logo_bytes(url)?;
        let png = widget_png_from_bytes(&raw)?;
        write_http_logo_cache(cache_dir, url, &png);
        Some(png)
    }

    fn widget_png_from_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
        let img = image::load_from_memory(bytes).ok()?;
        let max_side = 96u32;
        let (w, h) = (img.width(), img.height());
        let img = if w.max(h) > max_side {
            let scale = max_side as f64 / (w.max(h) as f64);
            let nw = ((w as f64) * scale).round().max(1.0) as u32;
            let nh = ((h as f64) * scale).round().max(1.0) as u32;
            img.resize(nw, nh, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        let mut out = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut out);
        img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        Some(out)
    }

    fn safe_icon_filename(id: &str) -> String {
        let s: String = id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        if s.is_empty() {
            "sub".into()
        } else {
            s
        }
    }

    fn materialize_logo_png(
        icons_dir: &Path,
        http_cache_dir: &Path,
        subscription_id: &str,
        logo: &str,
    ) -> Option<String> {
        let t = logo.trim();
        let lower = t.to_ascii_lowercase();
        let png: Vec<u8> = if lower.starts_with("http://") || lower.starts_with("https://") {
            http_logo_raster_cached(http_cache_dir, t)?
        } else {
            if t.is_empty() || !t.starts_with("data:") {
                return None;
            }
            if data_uri_header_is_svg(t) {
                return None;
            }
            let raw = decode_data_uri_bytes(t)?;
            widget_png_from_bytes(&raw)?
        };
        let fname = format!("{}.png", safe_icon_filename(subscription_id));
        let path = icons_dir.join(&fname);
        fs::write(&path, &png).ok()?;
        Some(format!("{}/{}", ICON_SUBDIR, fname))
    }

    pub(super) fn export_snapshot(inner: &AppStateInner) -> Result<(), String> {
        let subs = collect_upcoming_subscription_docs_for_widget(inner)?;
        let currencies: Vec<CurrencyDoc> =
            inner.table_list_typed(EntityTable::Currencies)?;
        let code_by_id: HashMap<String, String> = currencies
            .into_iter()
            .map(|c| {
                let code = c.code.trim();
                let iso = if code.is_empty() { "USD" } else { code };
                (c.id, iso.to_string())
            })
            .collect();

        let root = app_group_root().ok_or_else(|| {
            "App Group container unavailable (enable group.com.s00d.subly for the app)".to_string()
        })?;
        let dir = root.join(RELATIVE_WIDGET_DIR);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

        let logo_cache_dir = dir.join(HTTP_LOGO_CACHE_SUBDIR);
        fs::create_dir_all(&logo_cache_dir).map_err(|e| e.to_string())?;

        let icons_dir = dir.join(ICON_SUBDIR);
        let _ = fs::remove_dir_all(&icons_dir);
        fs::create_dir_all(&icons_dir).map_err(|e| e.to_string())?;

        let items: Vec<WidgetUpcomingItem> = subs
            .into_iter()
            .map(|s| {
                let currency_code = code_by_id
                    .get(&s.currency_id)
                    .cloned()
                    .unwrap_or_else(|| "USD".to_string());
                let icon_file =
                    materialize_logo_png(&icons_dir, &logo_cache_dir, &s.id, &s.logo);
                WidgetUpcomingItem {
                    id: s.id,
                    name: s.name,
                    next_payment: s.next_payment,
                    price: s.price,
                    currency_id: s.currency_id,
                    currency_code,
                    icon_file,
                }
            })
            .collect();

        let snapshot = WidgetSnapshot {
            updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            items,
        };

        let path = dir.join(UPCOMING_FILE);
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;

        reload_widget_timelines_if_swift_linked();

        Ok(())
    }
}

/// Refresh widget data from subscription state (no-op outside iOS).
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub fn export_ios_widget_snapshot_from_guard(inner: &crate::state::AppStateInner) {
    #[cfg(target_os = "ios")]
    {
        match ios_impl::export_snapshot(inner) {
            Ok(()) => {}
            Err(e) => eprintln!("[subly][widget] snapshot failed: {}", e),
        }
    }
    #[cfg(not(target_os = "ios"))]
    let _ = inner;
}

/// Same as [`export_ios_widget_snapshot_from_guard`] using managed app state.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub fn export_ios_widget_snapshot_from_app(app: &tauri::AppHandle) {
    #[cfg(target_os = "ios")]
    {
        use tauri::Manager;
        if let Some(state) = app.try_state::<crate::AppState>() {
            if let Ok(guard) = state.lock() {
                export_ios_widget_snapshot_from_guard(&guard);
            }
        }
    }
    #[cfg(not(target_os = "ios"))]
    let _ = app;
}
