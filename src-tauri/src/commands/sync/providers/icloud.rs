use super::super::{SyncConfig, SyncPayload};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::super::{decode_sync_payload, encode_sync_payload, SYNC_FILENAME, SYNC_ICLOUD_FOLDER};
use std::path::PathBuf;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::icloud_native;

pub fn descriptor() -> super::ProviderInfo {
    super::ProviderInfo {
        r#type: super::SyncProviderType::Icloud,
        name: "iCloud".to_string(),
        icon: "icloud".to_string(),
        fields: vec![],
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) fn icloud_subly_sync_dir() -> Option<PathBuf> {
    icloud_native::ubiquity_sync_subdir(SYNC_ICLOUD_FOLDER)
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub(crate) fn icloud_subly_sync_dir() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "macos")]
fn debug_dir_entries(path: &std::path::Path) -> String {
    match std::fs::read_dir(path) {
        Ok(rd) => {
            let mut names: Vec<String> = Vec::new();
            for ent in rd.flatten() {
                names.push(ent.file_name().to_string_lossy().into_owned());
                if names.len() >= 40 {
                    names.push("...".to_string());
                    break;
                }
            }
            names.sort();
            format!("[{}]", names.join(", "))
        }
        Err(e) => format!("<read_dir_error:{}>", e),
    }
}

#[cfg(target_os = "ios")]
fn debug_dir_entries(path: &std::path::Path) -> String {
    match std::fs::read_dir(path) {
        Ok(rd) => {
            let mut names: Vec<String> = Vec::new();
            for ent in rd.flatten() {
                names.push(ent.file_name().to_string_lossy().into_owned());
                if names.len() >= 40 {
                    names.push("...".to_string());
                    break;
                }
            }
            names.sort();
            format!("[{}]", names.join(", "))
        }
        Err(e) => format!("<read_dir_error:{}>", e),
    }
}

pub async fn download(_app: &tauri::AppHandle, _cfg: &SyncConfig) -> Result<Option<SyncPayload>, crate::errors::AppError> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        let Some(dir) = icloud_native::ubiquity_sync_subdir(SYNC_ICLOUD_FOLDER) else {
            #[cfg(target_os = "ios")]
            {
                return Err(crate::errors::AppError::from(
                    "iCloud ubiquity container is unavailable on iOS (check iCloud account, entitlements, and Cloud Documents capability)",
                ));
            }
            #[cfg(not(target_os = "ios"))]
            return Ok(None);
        };
        let path = dir.join(SYNC_FILENAME);
        #[cfg(target_os = "ios")]
        {
            let docs = dir.parent().map(|p| p.to_path_buf());
            eprintln!(
                "[subly][icloud][ios] read path={} exists={} is_file={}",
                path.display(),
                path.exists(),
                path.is_file()
            );
            eprintln!(
                "[subly][icloud][ios] sync_dir={} exists={} entries={}",
                dir.display(),
                dir.exists(),
                debug_dir_entries(&dir)
            );
            if let Some(docs_dir) = docs {
                eprintln!(
                    "[subly][icloud][ios] docs_dir={} exists={} entries={}",
                    docs_dir.display(),
                    docs_dir.exists(),
                    debug_dir_entries(&docs_dir)
                );
            }
        }
        let mut raw: Option<Vec<u8>> = None;
        let mut last_err: Option<crate::errors::AppError> = None;
        for _ in 0..4 {
            match icloud_native::coordinated_read_bytes(&path) {
                Ok(Some(b)) => {
                    raw = Some(b);
                    break;
                }
                Ok(None) => return Ok(None),
                Err(e) => {
                    #[cfg(target_os = "ios")]
                    eprintln!("[subly][icloud][ios] read attempt failed: {}", e);
                    if e.to_string().contains("still downloading") {
                        last_err = Some(e);
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        let raw = match raw {
            Some(v) => v,
            None => {
                return Err(last_err.unwrap_or_else(|| {
                    crate::errors::AppError::from("iCloud read failed")
                }))
            }
        };
        let payload = decode_sync_payload(&raw)?;
        Ok(Some(payload))
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        Ok(None)
    }
}

pub async fn upload(_app: &tauri::AppHandle, _cfg: &SyncConfig, _payload: &SyncPayload) -> Result<(), crate::errors::AppError> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        #[cfg(target_os = "macos")]
        let dir = icloud_native::ubiquity_sync_subdir(SYNC_ICLOUD_FOLDER).ok_or_else(|| {
            crate::errors::AppError::from(
                "iCloud app container folder not available on macOS (check iCloud account, code signing, and Cloud Documents entitlement)",
            )
        })?;
        #[cfg(target_os = "ios")]
        let dir = icloud_native::ubiquity_sync_subdir(SYNC_ICLOUD_FOLDER).ok_or_else(|| {
            crate::errors::AppError::from(
                "iCloud Drive folder not available (enable iCloud Drive on this device)",
            )
        })?;
        let raw = encode_sync_payload(_payload)?;
        let path = dir.join(SYNC_FILENAME);
        #[cfg(target_os = "ios")]
        {
            let docs = dir.parent().map(|p| p.to_path_buf());
            eprintln!(
                "[subly][icloud][ios] upload target={} dir_exists={} file_exists_before={}",
                path.display(),
                dir.exists(),
                path.exists()
            );
            eprintln!(
                "[subly][icloud][ios] upload sync_dir_entries_before={}",
                debug_dir_entries(&dir)
            );
            if let Some(docs_dir) = docs {
                eprintln!(
                    "[subly][icloud][ios] upload docs_dir_entries_before={}",
                    debug_dir_entries(&docs_dir)
                );
            }
        }
        #[cfg(target_os = "macos")]
        {
            let docs = dir.parent().map(|p| p.to_path_buf());
            eprintln!(
                "[subly][icloud][macos] upload target={} dir_exists={} file_exists_before={}",
                path.display(),
                dir.exists(),
                path.exists()
            );
            eprintln!(
                "[subly][icloud][macos] upload sync_dir_entries_before={}",
                debug_dir_entries(&dir)
            );
            if let Some(docs_dir) = docs {
                eprintln!(
                    "[subly][icloud][macos] upload docs_dir_entries_before={}",
                    debug_dir_entries(&docs_dir)
                );
            }
        }
        let res = icloud_native::coordinated_write_bytes(&path, &raw);
        #[cfg(target_os = "ios")]
        eprintln!(
            "[subly][icloud][ios] upload result={} file_exists_after={}",
            if res.is_ok() { "ok" } else { "err" },
            path.exists()
        );
        #[cfg(target_os = "macos")]
        eprintln!(
            "[subly][icloud][macos] upload result={} file_exists_after={}",
            if res.is_ok() { "ok" } else { "err" },
            path.exists()
        );
        res
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        Err(crate::errors::AppError::from("icloud not available"))
    }
}
