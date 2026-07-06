//! Writes a JSON snapshot for iOS / Android home screen widgets.
//! Widget UI only reads `widget/upcoming.json` (+ raster icons under `widget/icons/`).
//!
//! Heavy work runs on a background thread; `IN_PROGRESS` + `PENDING_REFRESH` coalesce bursts.

#[cfg(any(target_os = "ios", target_os = "android", test))]
mod shared {
    use crate::commands::subscriptions::collect_upcoming_subscription_docs_for_widget;
    use crate::models::{CurrencyDoc, SubscriptionDoc};
    use crate::state::AppStateInner;
    use crate::state::EntityTable;
    use base64::Engine;
    use serde::Serialize;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::OnceLock;
    use std::time::Duration;

    pub const RELATIVE_WIDGET_DIR: &str = "widget";
    const ICON_SUBDIR: &str = "icons";
    const HTTP_LOGO_CACHE_SUBDIR: &str = "logo_cache";
    const UPCOMING_FILE: &str = "upcoming.json";
    const UPCOMING_FILE_TMP: &str = "upcoming.json.tmp";
    const HTTP_LOGO_TIMEOUT: Duration = Duration::from_secs(4);

    static IN_PROGRESS: AtomicBool = AtomicBool::new(false);
    static PENDING_REFRESH: AtomicBool = AtomicBool::new(false);

    #[derive(Serialize)]
    pub struct WidgetUpcomingItem {
        pub id: String,
        pub name: String,
        #[serde(rename = "nextPayment")]
        pub next_payment: String,
        #[serde(with = "crate::money::serde_ipc")]
        pub price: rust_decimal::Decimal,
        #[serde(rename = "currencyId")]
        pub currency_id: String,
        #[serde(rename = "currencyCode")]
        pub currency_code: String,
        #[serde(rename = "iconFile", skip_serializing_if = "Option::is_none")]
        pub icon_file: Option<String>,
    }

    #[derive(Serialize)]
    pub struct WidgetSnapshot {
        #[serde(rename = "updatedAt")]
        pub updated_at: String,
        pub items: Vec<WidgetUpcomingItem>,
    }

    pub struct SnapshotJob {
        pub subs: Vec<SubscriptionDoc>,
        pub code_by_id: HashMap<String, String>,
    }

    pub fn collect_job(inner: &AppStateInner) -> Result<SnapshotJob, crate::errors::AppError> {
        let subs = collect_upcoming_subscription_docs_for_widget(inner)?;
        let currencies: Vec<CurrencyDoc> = inner.table_list_typed(EntityTable::Currencies)?;
        let code_by_id: HashMap<String, String> = currencies
            .into_iter()
            .map(|c| {
                let code = c.code.trim();
                let iso = if code.is_empty() { "USD" } else { code };
                (c.id, iso.to_string())
            })
            .collect();
        Ok(SnapshotJob { subs, code_by_id })
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

    fn http_agent() -> &'static ureq::Agent {
        static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
        AGENT.get_or_init(|| {
            ureq::Agent::config_builder()
                .timeout_global(Some(HTTP_LOGO_TIMEOUT))
                .build()
                .into()
        })
    }

    fn fetch_http_logo_bytes(url: &str) -> Option<Vec<u8>> {
        let mut resp = http_agent().get(url).call().ok()?;
        let buf = resp.body_mut().read_to_vec().ok()?;
        if buf.len() > 512 * 1024 {
            return None;
        }
        Some(buf)
    }

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

    fn atomic_write_file(path: &Path, contents: &[u8]) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| "atomic_write_file: missing parent directory".to_string())?;
        let tmp_path = parent.join(UPCOMING_FILE_TMP);
        fs::write(&tmp_path, contents).map_err(|e| e.to_string())?;
        if let Err(e) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e.to_string());
        }
        Ok(())
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

    /// `widget_dir` is the absolute path to the `widget/` folder.
    pub fn write_snapshot(widget_dir: &Path, job: &SnapshotJob) -> Result<(), String> {
        fs::create_dir_all(widget_dir).map_err(|e| e.to_string())?;

        let logo_cache_dir = widget_dir.join(HTTP_LOGO_CACHE_SUBDIR);
        fs::create_dir_all(&logo_cache_dir).map_err(|e| e.to_string())?;

        let icons_dir = widget_dir.join(ICON_SUBDIR);
        let _ = fs::remove_dir_all(&icons_dir);
        fs::create_dir_all(&icons_dir).map_err(|e| e.to_string())?;

        let items: Vec<WidgetUpcomingItem> = job
            .subs
            .iter()
            .map(|s| {
                let currency_code = job
                    .code_by_id
                    .get(&s.currency_id)
                    .cloned()
                    .unwrap_or_else(|| "USD".to_string());
                let icon_file =
                    materialize_logo_png(&icons_dir, &logo_cache_dir, &s.id, &s.logo);
                WidgetUpcomingItem {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    next_payment: s.next_payment.clone(),
                    price: s.price,
                    currency_id: s.currency_id.clone(),
                    currency_code,
                    icon_file,
                }
            })
            .collect();

        let snapshot = WidgetSnapshot {
            updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            items,
        };

        let path = widget_dir.join(UPCOMING_FILE);
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        atomic_write_file(&path, json.as_bytes())?;
        Ok(())
    }

    pub fn export_snapshot_async<F, G>(
        app_handle: tauri::AppHandle,
        initial: SnapshotJob,
        resolve_widget_dir: F,
        after_write: G,
    ) where
        F: Fn(&tauri::AppHandle) -> Result<PathBuf, String> + Send + Sync + 'static,
        G: Fn() + Send + Sync + 'static,
    {
        if IN_PROGRESS
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            PENDING_REFRESH.store(true, Ordering::Release);
            return;
        }

        std::thread::spawn(move || {
            use tauri::Manager;
            let mut job = initial;
            loop {
                match resolve_widget_dir(&app_handle) {
                    Ok(dir) => {
                        if let Err(e) = write_snapshot(&dir, &job) {
                            eprintln!("[subly][widget] snapshot failed: {}", e);
                        } else {
                            after_write();
                        }
                    }
                    Err(e) => eprintln!("[subly][widget] widget dir: {}", e),
                }
                if !PENDING_REFRESH.swap(false, Ordering::AcqRel) {
                    break;
                }
                let Some(state) = app_handle.try_state::<crate::AppState>() else {
                    break;
                };
                let Ok(guard) = state.lock() else { break };
                match collect_job(&guard) {
                    Ok(fresh) => job = fresh,
                    Err(e) => {
                        eprintln!("[subly][widget] refresh collect failed: {}", e);
                        break;
                    }
                }
            }
            IN_PROGRESS.store(false, Ordering::Release);
        });
    }
}

#[cfg(target_os = "ios")]
mod ios_impl {
    use super::shared::{self, SnapshotJob};
    use objc2::rc::autoreleasepool;
    use objc2_foundation::{NSFileManager, NSString};
    use std::path::PathBuf;

    const APP_GROUP_ID: &str = "group.com.s00d.subly";

    fn call_swift_widget_fn(symbol: &str) -> bool {
        use std::ffi::CString;
        let Ok(cstr) = CString::new(symbol) else {
            return false;
        };
        unsafe {
            let ptr = libc::dlsym(libc::RTLD_DEFAULT, cstr.as_ptr());
            if ptr.is_null() {
                return false;
            }
            let f: extern "C" fn() = std::mem::transmute(ptr);
            f();
            true
        }
    }

    fn reload_widget_timelines_if_swift_linked() {
        for name in ["_subly_reload_widget_timelines", "subly_reload_widget_timelines"] {
            if call_swift_widget_fn(name) {
                return;
            }
        }
    }

    pub(super) fn install_widget_reload_on_activate_if_swift_linked() {
        for name in [
            "_subly_install_widget_reload_on_activate",
            "subly_install_widget_reload_on_activate",
        ] {
            if call_swift_widget_fn(name) {
                return;
            }
        }
    }

    fn app_group_widget_dir() -> Result<PathBuf, String> {
        let root = autoreleasepool(|_pool| {
            let fm = NSFileManager::defaultManager();
            let url =
                fm.containerURLForSecurityApplicationGroupIdentifier(&NSString::from_str(APP_GROUP_ID))?;
            let path = url.path()?;
            Some(PathBuf::from(path.to_string()))
        })
        .ok_or_else(|| {
            "App Group container unavailable (enable group.com.s00d.subly for the app)".to_string()
        })?;
        Ok(root.join(shared::RELATIVE_WIDGET_DIR))
    }

    pub fn start_export(app: &tauri::AppHandle, job: SnapshotJob) {
        let app = app.clone();
        shared::export_snapshot_async(
            app,
            job,
            |_| app_group_widget_dir(),
            reload_widget_timelines_if_swift_linked,
        );
    }
}

#[cfg(target_os = "android")]
mod android_impl {
    use super::shared::SnapshotJob;
    use jni::objects::JClass;
    use jni::sys::{jint, JNI_VERSION_1_6};
    use jni::{JNIEnv, JavaVM};
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tauri::Manager;

    static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();

    pub fn store_java_vm(vm: JavaVM) {
        let _ = JAVA_VM.set(vm);
    }

    pub fn notify_widgets_updated() {
        let Some(vm) = JAVA_VM.get() else {
            return;
        };
        let Ok(mut env) = vm.attach_current_thread() else {
            return;
        };
        let Ok(class) = env.find_class("com/s00d/subly/widget/WidgetSnapshotNotifier") else {
            return;
        };
        let _ = env.call_static_method(class, "notifySnapshotUpdated", "()V", &[]);
    }

    fn widget_dir_for_app(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let base = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("app_local_data_dir: {e}"))?;
        std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
        Ok(base.join(super::shared::RELATIVE_WIDGET_DIR))
    }

    pub fn start_export(app: &tauri::AppHandle, job: SnapshotJob) {
        let app = app.clone();
        super::shared::export_snapshot_async(
            app,
            job,
            |handle| widget_dir_for_app(handle),
            notify_widgets_updated,
        );
    }

    #[no_mangle]
    pub extern "system" fn Java_com_s00d_subly_MainActivity_nativeInitWidgetBridge(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jint {
        match env.get_java_vm() {
            Ok(vm) => {
                store_java_vm(vm);
                JNI_VERSION_1_6
            }
            Err(_) => JNI_VERSION_1_6,
        }
    }
}

/// Refresh widget data from subscription state (no-op on desktop).
#[cfg_attr(not(any(target_os = "ios", target_os = "android")), allow(dead_code))]
pub fn export_widget_snapshot_from_app(app: &tauri::AppHandle) {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        use tauri::Manager;

        let Some(state) = app.try_state::<crate::AppState>() else {
            return;
        };
        let job = match state.lock() {
            Ok(guard) => match shared::collect_job(&guard) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[subly][widget] collect failed: {}", e);
                    return;
                }
            },
            Err(_) => return,
        };
        #[cfg(target_os = "ios")]
        ios_impl::start_export(app, job);
        #[cfg(target_os = "android")]
        android_impl::start_export(app, job);
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        let _ = app;
    }
}

/// Back-compat alias for existing call sites.
#[cfg_attr(not(any(target_os = "ios", target_os = "android")), allow(dead_code))]
pub fn export_ios_widget_snapshot_from_app(app: &tauri::AppHandle) {
    export_widget_snapshot_from_app(app);
}

#[cfg(target_os = "ios")]
pub fn install_ios_widget_reload_on_activate() {
    ios_impl::install_widget_reload_on_activate_if_swift_linked();
}

#[cfg(not(target_os = "ios"))]
#[allow(dead_code)]
pub fn install_ios_widget_reload_on_activate() {}

#[cfg(test)]
mod tests {
    use super::shared::{self, SnapshotJob, WidgetUpcomingItem};
    use crate::commands::subscriptions::{
        collect_upcoming_subscription_docs_for_widget, WIDGET_UPCOMING_DAYS,
        WIDGET_UPCOMING_LIMIT,
    };
    use crate::models::SubscriptionDoc;
    use crate::state::{save_app_data_typed, AppStateInner};
    use chrono::Local;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::fs;
    use std::str::FromStr;
    use std::sync::Arc;

    /// Mirrors iOS/Android widget JSON consumers: accept string or legacy number `price`.
    fn decode_price_like_widget_ui(value: &serde_json::Value) -> f64 {
        match value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.trim().parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn seeded_subscription_template() -> SubscriptionDoc {
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("seed doc");
        doc.subscriptions
            .into_iter()
            .next()
            .expect("restart-sensitive seed must include a subscription")
    }

    fn sample_job() -> SnapshotJob {
        let mut sub = seeded_subscription_template();
        sub.id = "sub-widget-1".to_string();
        sub.name = "Streaming".to_string();
        sub.next_payment = "2099-06-01".to_string();
        sub.price = Decimal::from_str("12.50").expect("decimal");
        let currency_id = sub.currency_id.clone();
        SnapshotJob {
            subs: vec![sub],
            code_by_id: HashMap::from([(currency_id, "USD".to_string())]),
        }
    }

    fn subscription_with_next_in_days(id: &str, days: i64, inactive: bool) -> SubscriptionDoc {
        let pay = Local::now().date_naive() + chrono::Duration::days(days);
        let mut sub = seeded_subscription_template();
        sub.id = id.to_string();
        sub.name = format!("Sub {id}");
        sub.next_payment = pay.format("%Y-%m-%d").to_string();
        sub.price = Decimal::from_str("1.00").expect("decimal");
        sub.inactive = inactive;
        sub
    }

    fn state_with_subscriptions(subs: Vec<SubscriptionDoc>) -> (tempfile::TempDir, AppStateInner) {
        let (dir, db) = crate::test_support::temp_db().expect("temp db");
        let mut doc = crate::test_support::base_seeded_doc().expect("seed");
        doc.subscriptions = subs;
        save_app_data_typed(&db, &doc, &crate::test_support::default_config()).expect("save");
        let inner = AppStateInner {
            db: Arc::new(db),
            app_data: doc,
        };
        (dir, inner)
    }

    #[test]
    fn widget_price_serializes_as_decimal_string() {
        let item = WidgetUpcomingItem {
            id: "x".into(),
            name: "Test".into(),
            next_payment: "2099-01-01".into(),
            price: Decimal::from_str("9.99").expect("decimal"),
            currency_id: "c1".into(),
            currency_code: "USD".into(),
            icon_file: None,
        };
        let v = serde_json::to_value(&item).expect("serialize");
        assert!(
            v.get("price")
                .and_then(|p| p.as_str())
                .is_some(),
            "widget IPC must emit price as string, got {:?}",
            v.get("price")
        );
        assert_eq!(v["price"], "9.99");
    }

    #[test]
    fn write_snapshot_writes_upcoming_json_with_string_prices() {
        let dir = tempfile::tempdir().expect("tempdir");
        let widget_dir = dir.path().join(shared::RELATIVE_WIDGET_DIR);
        shared::write_snapshot(&widget_dir, &sample_job()).expect("write");

        let path = widget_dir.join("upcoming.json");
        assert!(path.is_file(), "upcoming.json must exist");

        let tmp = widget_dir.join("upcoming.json.tmp");
        assert!(!tmp.exists(), "atomic write must not leave .tmp behind");

        let root: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("read"))
            .expect("json");
        let items = root["items"].as_array().expect("items array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["name"], "Streaming");
        assert!(
            items[0]["price"].is_string(),
            "written snapshot must use string prices"
        );
        assert_eq!(items[0]["price"], "12.50");
    }

    #[test]
    fn widget_consumer_accepts_string_and_legacy_number_price() {
        let string_json = serde_json::json!({
            "updatedAt": "2026-01-01T00:00:00Z",
            "items": [{
                "id": "a",
                "name": "A",
                "nextPayment": "2099-01-01",
                "price": "19.99",
                "currencyId": "c",
                "currencyCode": "USD"
            }]
        });
        let number_json = serde_json::json!({
            "updatedAt": "2026-01-01T00:00:00Z",
            "items": [{
                "id": "b",
                "name": "B",
                "nextPayment": "2099-01-01",
                "price": 4.5,
                "currencyId": "c",
                "currencyCode": "EUR"
            }]
        });

        for doc in [string_json, number_json] {
            let items = doc["items"].as_array().expect("items");
            assert_eq!(items.len(), 1);
            let amount = decode_price_like_widget_ui(&items[0]["price"]);
            assert!(amount > 0.0, "consumer decode failed for {:?}", items[0]["price"]);
        }
    }

    #[test]
    fn ios_xctest_fixture_uses_string_price_contract() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "ios-apple/SublyWidgetTests/Fixtures/upcoming_string_price.json",
        );
        let root: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("ios fixture")).expect("json");
        assert!(
            root["items"][0]["price"].is_string(),
            "shared iOS fixture must use string price (Rust serde_ipc)"
        );
        assert_eq!(root["items"][0]["price"], "12.50");
    }

    #[test]
    fn write_snapshot_output_is_decodable_by_consumer_rules() {
        let dir = tempfile::tempdir().expect("tempdir");
        let widget_dir = dir.path().join(shared::RELATIVE_WIDGET_DIR);
        shared::write_snapshot(&widget_dir, &sample_job()).expect("write");
        let path = widget_dir.join("upcoming.json");
        let root: serde_json::Value =
            serde_json::from_slice(&fs::read(path).expect("read")).expect("json");
        let items = root["items"].as_array().expect("items");
        assert!(!items.is_empty());
        let price = decode_price_like_widget_ui(&items[0]["price"]);
        assert!((price - 12.5).abs() < f64::EPSILON);
    }

    #[test]
    fn collect_upcoming_respects_window_inactive_and_limit() {
        let mut subs = vec![
            subscription_with_next_in_days("active-soon", 3, false),
            subscription_with_next_in_days("inactive-soon", 3, true),
            subscription_with_next_in_days("too-far", WIDGET_UPCOMING_DAYS + 5, false),
            subscription_with_next_in_days("past", -2, false),
        ];
        for i in 0..WIDGET_UPCOMING_LIMIT + 3 {
            subs.push(subscription_with_next_in_days(
                &format!("bulk-{i}"),
                5 + i as i64,
                false,
            ));
        }
        let (_dir, inner) = state_with_subscriptions(subs);
        let rows = collect_upcoming_subscription_docs_for_widget(&inner).expect("collect");
        assert!(rows.iter().all(|s| !s.inactive));
        assert!(rows.len() <= WIDGET_UPCOMING_LIMIT);
        assert!(rows.iter().any(|s| s.id == "active-soon"));
        assert!(!rows.iter().any(|s| s.id == "inactive-soon"));
        assert!(!rows.iter().any(|s| s.id == "too-far"));
        assert!(!rows.iter().any(|s| s.id == "past"));
    }

    #[test]
    fn collect_job_builds_currency_codes() {
        let (_dir, inner) = state_with_subscriptions(vec![subscription_with_next_in_days(
            "one", 7, false,
        )]);
        let job = shared::collect_job(&inner).expect("collect_job");
        assert!(!job.subs.is_empty());
        assert!(
            job.code_by_id.values().any(|c| !c.is_empty()),
            "currency ISO codes should be populated from catalog"
        );
    }
}
