use base64::Engine as _;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tauri::Emitter;
use tauri::State;
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use crate::AppState;
use crate::models::{AppConfigDoc, AppDataDoc};
use crate::state::{read_singleton_bin_typed, T2_CONFIG};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub ok: bool,
    pub message: Option<String>,
    pub imported_count: Option<usize>,
}

const STATUS_INVALID_ARCHIVE: &str = "invalid_archive";
const STATUS_PATH_REQUIRED: &str = "path_required";

fn apply_imported_snapshot(
    guard: &mut crate::state::AppStateInner,
    data: &AppDataDoc,
    config: &AppConfigDoc,
) -> Result<(), crate::errors::AppError> {
    guard.apply_snapshot_typed_with_config(data, config)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPathSet {
    pub subly_backup: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPathPresets {
    pub documents: Option<ExportPathSet>,
    pub downloads: Option<ExportPathSet>,
}

fn parse_file_path(path: &str) -> Result<FilePath, crate::errors::AppError> {
    FilePath::from_str(path).map_err(|e| crate::errors::AppError::Message(e.to_string()))
}

fn require_path(path: Option<String>) -> Result<FilePath, ExportResult> {
    let Some(raw_path) = path else {
        return Err(ExportResult {
            ok: false,
            message: Some(STATUS_PATH_REQUIRED.to_string()),
            imported_count: None,
        });
    };
    match parse_file_path(&raw_path) {
        Ok(fp) => Ok(fp),
        Err(_) => Err(ExportResult {
            ok: false,
            message: Some(STATUS_PATH_REQUIRED.to_string()),
            imported_count: None,
        }),
    }
}

fn build_path_set(base: &Path) -> ExportPathSet {
    let date = chrono::Utc::now().date_naive().to_string();
    let to_s = |p: PathBuf| p.to_string_lossy().into_owned();
    ExportPathSet {
        subly_backup: to_s(base.join(format!("subly-backup-{}.subly", date))),
    }
}

#[tauri::command]
pub fn export_get_path_presets() -> Result<ExportPathPresets, crate::errors::AppError> {
    Ok(ExportPathPresets {
        documents: dirs::document_dir().map(|p| build_path_set(&p)),
        downloads: dirs::download_dir().map(|p| build_path_set(&p)),
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPathArgs {
    pub path: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSublyBytesArgs {
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub fn export_subly_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    args: Option<ExportPathArgs>,
) -> Result<ExportResult, crate::errors::AppError> {
    let fp = match require_path(args.and_then(|a| a.path)) {
        Ok(p) => p,
        Err(e) => return Ok(e),
    };

    let (data, config) = {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        let data = guard.app_data.clone();
        let config: AppConfigDoc =
            read_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, AppConfigDoc::default())?;
        (data, config)
    };

    let bytes = export_build_subly_archive_with_config(data, config)?;

    let mut open_opts = OpenOptions::new();
    open_opts
        .read(false)
        .write(true)
        .create(true)
        .truncate(true);
    let mut file = app
        .fs()
        .open(fp, open_opts.clone())
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(ExportResult {
        ok: true,
        message: None,
        imported_count: None,
    })
}

#[tauri::command]
pub fn import_subly_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    args: Option<ExportPathArgs>,
) -> Result<ExportResult, crate::errors::AppError> {
    let fp = match require_path(args.and_then(|a| a.path)) {
        Ok(p) => p,
        Err(e) => return Ok(e),
    };
    let bytes = app.fs().read(fp).map_err(|e| e.to_string())?;
    import_apply_subly_bytes(&app, &state, bytes)
}

#[tauri::command]
pub fn import_subly_backup_bytes(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    args: ImportSublyBytesArgs,
) -> Result<ExportResult, crate::errors::AppError> {
    import_apply_subly_bytes(&app, &state, args.bytes)
}

fn import_apply_subly_bytes(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    bytes: Vec<u8>,
) -> Result<ExportResult, crate::errors::AppError> {
    let parsed = import_parse_subly_archive_with_config(bytes).map_err(|e| e.to_string())?;
    let Some((data, cfg)) = parsed else {
        return Ok(ExportResult {
            ok: false,
            message: Some(STATUS_INVALID_ARCHIVE.to_string()),
            imported_count: None,
        });
    };
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    apply_imported_snapshot(&mut guard, &data, &cfg)?;
    let _ = app.emit(
        "app:data-changed",
        serde_json::json!({ "entity": "appData", "action": "import" }),
    );
    Ok(ExportResult {
        ok: true,
        message: None,
        imported_count: None,
    })
}

fn is_base64_icon(icon: &str) -> bool {
    icon.starts_with("data:")
}

fn data_uri_to_bytes(data_uri: &str) -> Result<Vec<u8>, crate::errors::AppError> {
    let base64_part = data_uri
        .split(',')
        .nth(1)
        .ok_or_else(|| crate::errors::AppError::from("invalid data uri"))?;
    base64::engine::general_purpose::STANDARD
        .decode(base64_part)
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))
}

fn bytes_to_data_uri(bytes: &[u8], filename: &str) -> String {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("png")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime, b64)
}

#[tauri::command]
#[allow(dead_code)]
pub fn export_build_subly_archive(data: AppDataDoc) -> Result<Vec<u8>, crate::errors::AppError> {
    let mut cfg = AppConfigDoc::default();
    cfg.initialized = true;
    export_build_subly_archive_with_config(data, cfg)
}

fn export_build_subly_archive_with_config(data: AppDataDoc, config: AppConfigDoc) -> Result<Vec<u8>, crate::errors::AppError> {
    let mut data = data;
    let mut icons: HashMap<String, String> = HashMap::new();
    let mut idx = 0usize;

    let mut remember_icon = |raw: &str| -> String {
        if let Some((name, _)) = icons.iter().find(|(_, v)| v.as_str() == raw) {
            return name.clone();
        }
        let ext = raw
            .strip_prefix("data:image/")
            .and_then(|s| s.split(';').next())
            .unwrap_or("png")
            .replace("jpeg", "jpg");
        let name = format!("icon_{}.{}", idx, ext);
        idx += 1;
        icons.insert(name.clone(), raw.to_string());
        name
    };

    for s in &mut data.subscriptions {
        if is_base64_icon(&s.logo) {
            let name = remember_icon(&s.logo);
            s.logo = format!("icons/{}", name);
        }
    }

    let mut buf = Cursor::new(Vec::<u8>::new());
    let mut zip = ZipWriter::new(&mut buf);
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("data.json", options).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({
        "schemaVersion": 2,
        "appData": data,
        "appConfig": config,
    });
    zip.write_all(serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?.as_bytes())
        .map_err(|e| e.to_string())?;

    for (name, data_uri) in &icons {
        zip.start_file(format!("icons/{}", name), options)
            .map_err(|e| e.to_string())?;
        let bytes = data_uri_to_bytes(data_uri)?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    zip.start_file("meta.json", options).map_err(|e| e.to_string())?;
    let meta = serde_json::json!({
      "version": 1,
      "app": "subly",
      "exportedAt": chrono::Utc::now().to_rfc3339(),
      "iconCount": icons.len(),
    });
    zip.write_all(serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?.as_bytes())
        .map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

fn validate_import_payload(data: &AppDataDoc) -> bool {
    let has_core_lists = !data.categories.is_empty()
        && !data.currencies.is_empty()
        && !data.household.is_empty()
        && !data.payment_methods.is_empty();
    if !has_core_lists {
        return false;
    }
    let valid_currency_ids: std::collections::HashSet<&str> =
        data.currencies.iter().map(|c| c.id.as_str()).collect();
    let main_currency_ok = !data.settings.main_currency_id.trim().is_empty()
        && valid_currency_ids.contains(data.settings.main_currency_id.as_str());
    if !main_currency_ok {
        return false;
    }
    data.currencies.iter().all(|c| !c.code.trim().is_empty())
}

#[tauri::command]
#[allow(dead_code)]
pub fn import_parse_subly_archive(bytes: Vec<u8>) -> Result<Option<AppDataDoc>, crate::errors::AppError> {
    let parsed = import_parse_subly_archive_with_config(bytes)?;
    Ok(parsed.map(|(data, _)| data))
}

/// Only for **importing old backup JSON** (not runtime DB): map legacy `date` / Y-M-D fields → `createdAt`.
fn normalize_legacy_expense_dates_in_import_json(value: &mut serde_json::Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    let Some(expenses) = obj.get_mut("expenses").and_then(|e| e.as_array_mut()) else {
        return;
    };
    for exp in expenses {
        let Some(o) = exp.as_object_mut() else {
            continue;
        };
        let has_created = o
            .get("createdAt")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !has_created {
            let from_ymd: Option<String> = (|| {
                let y = o.get("dateYear")?.as_i64()? as i32;
                let m = o.get("dateMonth")?.as_i64()? as u32;
                let d = o.get("dateDay")?.as_i64()? as u32;
                crate::models::ymd_to_utc_noon_rfc3339(y, m, d).ok()
            })();
            let from_date = o
                .get("date")
                .and_then(|v| v.as_str())
                .and_then(|s| crate::models::normalize_expense_timestamp(s).ok());
            if let Some(s) = from_ymd.or(from_date) {
                o.insert("createdAt".into(), serde_json::Value::String(s));
            }
        }
        o.remove("dateYear");
        o.remove("dateMonth");
        o.remove("dateDay");
        o.remove("date");
    }
}

fn parse_import_payload_json(raw_json: &str) -> Result<(AppDataDoc, AppConfigDoc), crate::errors::AppError> {
    let raw_value: serde_json::Value = serde_json::from_str(raw_json).map_err(|e| e.to_string())?;
    let data: AppDataDoc = if let Some(app_data) = raw_value.get("appData") {
        let mut v = app_data.clone();
        normalize_legacy_expense_dates_in_import_json(&mut v);
        serde_json::from_value(v).map_err(|e| e.to_string())?
    } else {
        let mut v = raw_value.clone();
        normalize_legacy_expense_dates_in_import_json(&mut v);
        serde_json::from_value(v).map_err(|e| e.to_string())?
    };
    let mut config: AppConfigDoc = if let Some(app_cfg) = raw_value.get("appConfig") {
        serde_json::from_value(app_cfg.clone()).map_err(|e| e.to_string())?
    } else {
        serde_json::from_value(raw_value.clone()).unwrap_or_default()
    };
    config.initialized = true;
    Ok((data, config))
}

fn import_parse_subly_archive_with_config(bytes: Vec<u8>) -> Result<Option<(AppDataDoc, AppConfigDoc)>, crate::errors::AppError> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;
    let mut data_json = String::new();
    {
        let mut data_file = match archive.by_name("data.json") {
            Ok(f) => f,
            Err(_) => return Ok(None),
        };
        data_file
            .read_to_string(&mut data_json)
            .map_err(|e| e.to_string())?;
    }
    let (mut data, config) = parse_import_payload_json(&data_json)?;

    let mut icons = HashMap::<String, String>::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        if !name.starts_with("icons/") || file.is_dir() {
            continue;
        }
        let mut raw = Vec::new();
        file.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        icons.insert(name.clone(), bytes_to_data_uri(&raw, &name));
    }

    for s in &mut data.subscriptions {
        if let Some(inline) = icons.get(&s.logo) {
            s.logo = inline.clone();
        }
    }
    Ok(if validate_import_payload(&data) {
        Some((data, config))
    } else {
        None
    })
}


#[cfg(test)]
mod tests {
    use super::{export_build_subly_archive_with_config, import_parse_subly_archive, parse_import_payload_json};
    use crate::state::{load_app_data_typed, save_app_data_typed};

    #[test]
    fn subly_archive_roundtrip_preserves_sensitive_fields_after_restart_load() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let mut cfg = crate::models::AppConfigDoc::default();
        cfg.initialized = true;
        cfg.telegram_enabled = true;
        cfg.telegram_chat_id = "12345".to_string();
        let archive = export_build_subly_archive_with_config(before.clone(), cfg).expect("build archive");
        let parsed = import_parse_subly_archive(archive)
            .expect("parse archive")
            .expect("archive contains data");

        let cfg = crate::test_support::default_config();
        save_app_data_typed(&db, &parsed, &cfg).expect("save parsed");
        let (after_restart, _) = load_app_data_typed(&db).expect("reload");
        crate::test_support::assert_core_invariants(&before, &after_restart);
    }

    #[test]
    fn json_bytes_roundtrip_preserves_sensitive_fields_after_restart_load() {
        let (_dir, db) = crate::test_support::temp_db().expect("temp db");
        let before = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let json = serde_json::to_vec(&before).expect("serialize");
        let parsed: crate::models::AppDataDoc = serde_json::from_slice(&json).expect("deserialize");

        let cfg = crate::test_support::default_config();
        save_app_data_typed(&db, &parsed, &cfg).expect("save parsed");
        let (after_restart, _) = load_app_data_typed(&db).expect("reload");
        crate::test_support::assert_core_invariants(&before, &after_restart);
    }

    #[test]
    fn invalid_archive_returns_none() {
        let invalid = vec![1_u8, 2_u8, 3_u8, 4_u8];
        let result = import_parse_subly_archive(invalid);
        assert!(result.is_err() || matches!(result, Ok(None)), "invalid archive should not parse as data");
    }

    #[test]
    fn validate_import_payload_rejects_empty_or_incomplete_core() {
        let mut doc = crate::test_support::base_seeded_doc().expect("seed");
        doc.currencies.clear();
        assert!(!super::validate_import_payload(&doc), "must reject empty currencies");

        let mut doc2 = crate::test_support::base_seeded_doc().expect("seed");
        doc2.settings.main_currency_id = "missing".to_string();
        assert!(
            !super::validate_import_payload(&doc2),
            "must reject unknown main_currency_id"
        );
    }

    #[test]
    fn legacy_flatten_payload_restores_app_config_fields() {
        let payload = serde_json::json!({
            "subscriptions": [],
            "expenses": [],
            "categories": [],
            "currencies": [],
            "household": [],
            "paymentMethods": [],
            "tags": [],
            "settings": {"budget": 0.0, "mainCurrencyId": "", "currencyUpdateTargets": [], "rateHistoryDays": 90},
            "initialized": true,
            "ratesProvider": "ecb",
            "telegramEnabled": true,
            "telegramChatId": "999",
            "telegramBotToken": "token"
        });
        let (data, cfg) = parse_import_payload_json(&serde_json::to_string(&payload).expect("serialize")).expect("parse");
        assert!(data.subscriptions.is_empty(), "data should parse from legacy payload");
        assert!(cfg.initialized, "config initialized should be restored");
        assert_eq!(cfg.rates_provider, "ecb", "ratesProvider should be restored");
        assert!(cfg.telegram_enabled, "telegramEnabled should be restored");
        assert_eq!(cfg.telegram_chat_id, "999", "telegramChatId should be restored");
    }
}
