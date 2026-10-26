//! iCloud access on Apple platforms via **ubiquity container** (`NSFileManager::URLForUbiquityContainerIdentifier`)
//! and coordinated I/O (`NSFileCoordinator`). Avoids hard-coded `com~apple~CloudDocs` paths on iOS and
//! matches App Store / sandbox expectations.

use core::ptr::NonNull;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::autoreleasepool;
use objc2_foundation::{
    NSFileCoordinator, NSFileCoordinatorReadingOptions, NSFileCoordinatorWritingOptions, NSFileManager, NSError,
    NSString, NSURL, NSURLUbiquitousItemDownloadingErrorKey, NSURLUbiquitousItemDownloadingStatusCurrent,
    NSURLUbiquitousItemDownloadingStatusDownloaded, NSURLUbiquitousItemDownloadingStatusKey,
    NSURLUbiquitousItemIsDownloadingKey,
};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;

use super::super::icloud_resolve::resolve_icloud_sync_file_from_names;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::commands::sync::config::APP_ICLOUD_CONTAINER_ID;

fn path_to_file_url(path: &Path) -> Result<objc2::rc::Retained<NSURL>, crate::errors::AppError> {
    let s = path
        .to_str()
        .ok_or_else(|| crate::errors::AppError::from("non-utf8 path"))?;
    Ok(NSURL::fileURLWithPath(&NSString::from_str(s)))
}

static CONTAINER_ROOT_CACHE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

/// Clear cached ubiquity container root (e.g. after account switch). Reserved for future hooks.
#[allow(dead_code)]
pub(crate) fn invalidate_container_cache() {
    if let Some(lock) = CONTAINER_ROOT_CACHE.get() {
        if let Ok(mut guard) = lock.lock() {
            *guard = None;
        }
    }
}

/// Mirrors `FileManager.ubiquityIdentityToken` from tauri-plugin-icloud-container.
pub(crate) fn is_icloud_user_signed_in() -> bool {
    autoreleasepool(|_pool| NSFileManager::defaultManager().ubiquityIdentityToken().is_some())
}

pub(crate) fn icloud_availability_error_key() -> &'static str {
    if is_icloud_user_signed_in() {
        "sync_icloud_unavailable"
    } else {
        "sync_icloud_not_signed_in"
    }
}

fn resolve_container_root_url() -> Option<objc2::rc::Retained<NSURL>> {
    autoreleasepool(|_pool| {
        let fm = NSFileManager::defaultManager();
        fm.URLForUbiquityContainerIdentifier(Some(&NSString::from_str(APP_ICLOUD_CONTAINER_ID)))
    })
}

fn cached_container_root() -> Option<PathBuf> {
    let lock = CONTAINER_ROOT_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(path) = guard.as_ref() {
            return Some(path.clone());
        }
    }
    let container = resolve_container_root_url()?;
    let path = container.path().map(|p| PathBuf::from(p.to_string()))?;
    if let Ok(mut guard) = lock.lock() {
        *guard = Some(path.clone());
    }
    Some(path)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcloudDebugState {
    pub signed_in: bool,
    pub container_path: Option<String>,
    pub sync_dir: Option<String>,
    pub dir_entries: Vec<String>,
    pub resolved_sync_file: Option<String>,
}

pub(crate) fn icloud_debug_state(sync_folder: &str) -> IcloudDebugState {
    let signed_in = is_icloud_user_signed_in();
    let sync_dir = ubiquity_sync_subdir(sync_folder);
    let container_path = cached_container_root().map(|p| p.to_string_lossy().into_owned());
    let dir_entries = sync_dir
        .as_ref()
        .map(|d| list_directory_names(d))
        .unwrap_or_default();
    let resolved_sync_file = sync_dir
        .as_ref()
        .and_then(|d| resolve_icloud_sync_file(d))
        .map(|p| p.to_string_lossy().into_owned());
    IcloudDebugState {
        signed_in,
        container_path,
        sync_dir: sync_dir.map(|p| p.to_string_lossy().into_owned()),
        dir_entries,
        resolved_sync_file,
    }
}

/// `Documents/<relative_folder>` inside the app’s iCloud container (see entitlements).
pub(crate) fn ubiquity_sync_subdir(relative_folder: &str) -> Option<PathBuf> {
    autoreleasepool(|_pool| {
        let container = resolve_container_root_url()?;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(cpath) = container.path() {
            #[cfg(target_os = "ios")]
            eprintln!("[subly][icloud][ios] container={}", cpath);
            #[cfg(target_os = "macos")]
            eprintln!("[subly][icloud][macos] container={}", cpath);
        }
        if let Some(cpath) = container.path() {
            if let Ok(mut guard) = CONTAINER_ROOT_CACHE
                .get_or_init(|| Mutex::new(None))
                .lock()
            {
                *guard = Some(PathBuf::from(cpath.to_string()));
            }
        }
        let fm = NSFileManager::defaultManager();
        let docs = container.URLByAppendingPathComponent(&NSString::from_str("Documents"))?;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(dpath) = docs.path() {
            #[cfg(target_os = "ios")]
            eprintln!("[subly][icloud][ios] docs={}", dpath);
            #[cfg(target_os = "macos")]
            eprintln!("[subly][icloud][macos] docs={}", dpath);
        }
        let _ = unsafe {
            fm.createDirectoryAtURL_withIntermediateDirectories_attributes_error(
                docs.as_ref(),
                true,
                None,
            )
        };
        let folder_ns = NSString::from_str(relative_folder);
        let sync_url = docs.URLByAppendingPathComponent(&folder_ns)?;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(spath) = sync_url.path() {
            #[cfg(target_os = "ios")]
            eprintln!("[subly][icloud][ios] sync_dir={}", spath);
            #[cfg(target_os = "macos")]
            eprintln!("[subly][icloud][macos] sync_dir={}", spath);
        }
        let _ = unsafe {
            fm.createDirectoryAtURL_withIntermediateDirectories_attributes_error(
                sync_url.as_ref(),
                true,
                None,
            )
        };
        let path_ns = sync_url.path()?;
        Some(PathBuf::from(path_ns.to_string()))
    })
}

/// Directory listing via NSFileManager (may include iCloud placeholders not visible to `std::fs::read_dir`).
pub(crate) fn list_directory_names(dir: &Path) -> Vec<String> {
    autoreleasepool(|_pool| {
        let Some(s) = dir.to_str() else {
            return Vec::new();
        };
        let fm = NSFileManager::defaultManager();
        let Ok(arr) = fm.contentsOfDirectoryAtPath_error(&NSString::from_str(s)) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for i in 0..arr.count() {
            out.push(arr.objectAtIndex(i).to_string());
        }
        out.sort();
        out
    })
}

/// Pick the sync blob inside the ubiquity folder (current v2 name, legacy v1, or any `subly-sync*`).
/// Prefer `NSFileManager` listing — it sees iCloud placeholders that `Path::is_file` misses.
pub(crate) fn resolve_icloud_sync_file(dir: &Path) -> Option<PathBuf> {
    let names = list_directory_names(dir);
    if let Some(path) = resolve_icloud_sync_file_from_names(dir, &names) {
        return Some(path);
    }
    let primary = dir.join(crate::commands::sync::config::SYNC_FILENAME);
    if primary.is_file() {
        return Some(primary);
    }
    let legacy = dir.join(crate::commands::sync::config::SYNC_FILENAME_LEGACY_JSON);
    if legacy.is_file() {
        return Some(legacy);
    }
    None
}

enum LocalItemState {
    Ready,
    Missing,
}

fn ubiquitous_item_downloaded(url: &NSURL, path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    autoreleasepool(|_pool| {
        let mut status_value: Option<Retained<AnyObject>> = None;
        let status_ok = unsafe {
            url.getResourceValue_forKey_error(
                &mut status_value,
                NSURLUbiquitousItemDownloadingStatusKey,
            )
            .is_ok()
        };
        if status_ok {
            if let Some(obj) = status_value {
                if let Some(status) = obj.downcast_ref::<NSString>() {
                    let s = status.to_string();
                    let downloaded = unsafe { NSURLUbiquitousItemDownloadingStatusDownloaded.to_string() };
                    let current = unsafe { NSURLUbiquitousItemDownloadingStatusCurrent.to_string() };
                    if s == downloaded || s == current {
                        return path.is_file();
                    }
                }
            }
        }
        let mut downloading: Option<Retained<AnyObject>> = None;
        let _ = unsafe {
            url.getResourceValue_forKey_error(&mut downloading, NSURLUbiquitousItemIsDownloadingKey)
        };
        let mut dl_err: Option<Retained<AnyObject>> = None;
        if unsafe {
            url.getResourceValue_forKey_error(&mut dl_err, NSURLUbiquitousItemDownloadingErrorKey)
                .is_ok()
        }
            && dl_err.is_some() {
                return false;
            }
        false
    })
}

fn wait_for_ubiquitous_download(path: &Path, url: &NSURL, attempts: usize, step: Duration) -> bool {
    for _ in 0..attempts {
        if ubiquitous_item_downloaded(url, path) {
            return true;
        }
        std::thread::sleep(step);
    }
    false
}

fn prepare_ubiquitous_item_local(path: &Path) -> Result<LocalItemState, crate::errors::AppError> {
    autoreleasepool(|_pool| {
        if path.is_file() {
            return Ok(LocalItemState::Ready);
        }

        let url = path_to_file_url(path)?;
        let fm = NSFileManager::defaultManager();
        let is_ubiquitous = fm.isUbiquitousItemAtURL(&url);

        // On a second device the file may exist in iCloud metadata before a local placeholder appears.
        // Always try startDownloading — even when isUbiquitous is false for a not-yet-materialized path.
        let dl_res = fm.startDownloadingUbiquitousItemAtURL_error(&url);
        if wait_for_ubiquitous_download(path, &url, 36, Duration::from_millis(250)) {
            return Ok(LocalItemState::Ready);
        }

        match dl_res {
            Ok(()) => Err(crate::errors::AppError::from("sync_icloud_materializing")),
            Err(_) if is_ubiquitous => Err(crate::errors::AppError::from("sync_icloud_materializing")),
            Err(_) if path.parent().is_some_and(|d| resolve_icloud_sync_file(d).is_some()) => {
                Err(crate::errors::AppError::from("sync_icloud_materializing"))
            }
            Err(_) => Ok(LocalItemState::Missing),
        }
    })
}

/// `None` until the coordinator invokes the accessor (distinguishes “no callback” from “file missing”).
type ReadOutcome = Option<Result<Option<String>, crate::errors::AppError>>;
type ReadBytesOutcome = Option<Result<Option<Vec<u8>>, crate::errors::AppError>>;
type WriteOutcome = Option<Result<(), crate::errors::AppError>>;

pub(crate) fn coordinated_read_string(path: &Path) -> Result<Option<String>, crate::errors::AppError> {
    autoreleasepool(|_pool| {
        let url = path_to_file_url(path)?;
        let coordinator = NSFileCoordinator::new();
        let out: Arc<Mutex<ReadOutcome>> = Arc::new(Mutex::new(None));
        let out_cb = Arc::clone(&out);
        let block = RcBlock::new(move |new_url: NonNull<NSURL>| {
            let u = unsafe { new_url.as_ref() };
            let Some(p) = u.path() else {
                *out_cb.lock().expect("lock") = Some(Err("missing path from NSFileCoordinator".into()));
                return;
            };
            let pb = PathBuf::from(p.to_string());
            let done = match std::fs::read_to_string(&pb) {
                Ok(s) => Ok(Some(s)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            };
            *out_cb.lock().expect("lock") = Some(done);
        });
        let mut coord_err: Option<objc2::rc::Retained<NSError>> = None;
        coordinator.coordinateReadingItemAtURL_options_error_byAccessor(
            &url,
            NSFileCoordinatorReadingOptions::WithoutChanges,
            Some(&mut coord_err),
            &block,
        );
        drop(block);
        if let Some(e) = coord_err {
            return Err(crate::errors::AppError::Message(
                e.localizedDescription().to_string(),
            ));
        }
        let inner = Arc::try_unwrap(out)
            .map_err(|_| {
                crate::errors::AppError::from("coordinator internal: Arc still held after drop(block)")
            })?
            .into_inner()
            .map_err(|_| {
                crate::errors::AppError::from("coordinator result poisoned")
            })?;
        match inner {
            None => Err(crate::errors::AppError::from(
                "coordinator callback did not run (NSFileCoordinator did not invoke the accessor)",
            )),
            Some(Err(e)) => Err(e),
            Some(Ok(v)) => Ok(v),
        }
    })
}

pub(crate) fn coordinated_read_bytes(path: &Path) -> Result<Option<Vec<u8>>, crate::errors::AppError> {
    match prepare_ubiquitous_item_local(path) {
        Ok(LocalItemState::Missing) => return Ok(None),
        Ok(LocalItemState::Ready) => {}
        Err(e) => return Err(e),
    }
    autoreleasepool(|_pool| {
        let url = path_to_file_url(path)?;
        let coordinator = NSFileCoordinator::new();
        let out: Arc<Mutex<ReadBytesOutcome>> = Arc::new(Mutex::new(None));
        let out_cb = Arc::clone(&out);
        let block = RcBlock::new(move |new_url: NonNull<NSURL>| {
            let u = unsafe { new_url.as_ref() };
            let Some(p) = u.path() else {
                *out_cb.lock().expect("lock") = Some(Err("missing path from NSFileCoordinator".into()));
                return;
            };
            let pb = PathBuf::from(p.to_string());
            let done = match std::fs::read(&pb) {
                Ok(b) => Ok(Some(b)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            };
            *out_cb.lock().expect("lock") = Some(done);
        });
        let mut coord_err: Option<objc2::rc::Retained<NSError>> = None;
        coordinator.coordinateReadingItemAtURL_options_error_byAccessor(
            &url,
            NSFileCoordinatorReadingOptions::WithoutChanges,
            Some(&mut coord_err),
            &block,
        );
        drop(block);
        if let Some(e) = coord_err {
            return Err(crate::errors::AppError::Message(
                e.localizedDescription().to_string(),
            ));
        }
        let inner = Arc::try_unwrap(out)
            .map_err(|_| {
                crate::errors::AppError::from("coordinator internal: Arc still held after drop(block)")
            })?
            .into_inner()
            .map_err(|_| {
                crate::errors::AppError::from("coordinator result poisoned")
            })?;
        match inner {
            None => Err(crate::errors::AppError::from(
                "coordinator callback did not run (NSFileCoordinator did not invoke the accessor)",
            )),
            Some(Err(e)) => Err(e),
            Some(Ok(v)) => Ok(v),
        }
    })
}

pub(crate) fn coordinated_write_bytes(path: &Path, contents: &[u8]) -> Result<(), crate::errors::AppError> {
    autoreleasepool(|_pool| {
        let url = path_to_file_url(path)?;
        if let Some(parent) = path.parent() {
            let parent_url = path_to_file_url(parent)?;
            let fm = NSFileManager::defaultManager();
            let _ = unsafe {
                fm.createDirectoryAtURL_withIntermediateDirectories_attributes_error(
                    parent_url.as_ref(),
                    true,
                    None,
                )
            };
        }
        let tmp_file_name = format!(
            "{}.tmp",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(crate::commands::sync::config::SYNC_FILENAME)
        );
        let coordinator = NSFileCoordinator::new();
        let data = contents.to_vec();
        let out: Arc<Mutex<WriteOutcome>> = Arc::new(Mutex::new(None));
        let out_cb = Arc::clone(&out);
        let block = RcBlock::new(move |new_url: NonNull<NSURL>| {
            let u = unsafe { new_url.as_ref() };
            let Some(p) = u.path() else {
                *out_cb.lock().expect("lock") = Some(Err("missing path from NSFileCoordinator".into()));
                return;
            };
            let pb = PathBuf::from(p.to_string());
            let tmp_pb = pb.with_file_name(&tmp_file_name);
            let res = (|| -> Result<(), crate::errors::AppError> {
                {
                    let mut f = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&tmp_pb)?;
                    std::io::Write::write_all(&mut f, &data)?;
                    f.sync_all()?;
                }
                std::fs::rename(&tmp_pb, &pb)?;
                Ok(())
            })();
            *out_cb.lock().expect("lock") = Some(res);
        });
        let mut coord_err: Option<objc2::rc::Retained<NSError>> = None;
        coordinator.coordinateWritingItemAtURL_options_error_byAccessor(
            &url,
            NSFileCoordinatorWritingOptions::ForReplacing,
            Some(&mut coord_err),
            &block,
        );
        drop(block);
        if let Some(e) = coord_err {
            return Err(crate::errors::AppError::Message(
                e.localizedDescription().to_string(),
            ));
        }
        let inner = Arc::try_unwrap(out)
            .map_err(|_| {
                crate::errors::AppError::from("coordinator internal: Arc still held after drop(block)")
            })?
            .into_inner()
            .map_err(|_| {
                crate::errors::AppError::from("coordinator result poisoned")
            })?;
        match inner {
            None => Err(crate::errors::AppError::from(
                "coordinator callback did not run (NSFileCoordinator did not invoke the accessor)",
            )),
            Some(Err(e)) => Err(e),
            Some(Ok(())) => Ok(()),
        }
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn legacy_clouddocs_sync_dir(folder: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Mobile Documents")
            .join(crate::commands::sync::config::SYNC_LEGACY_CLOUDDOCS_SEGMENT)
            .join(folder),
    )
}

/// One-shot: copy SS2 blob from legacy `com~apple~CloudDocs/Subly-v1` into ubiquity when ubiquity is empty.
#[cfg(target_os = "macos")]
pub(crate) fn migrate_legacy_clouddocs_if_needed(folder: &str) -> Result<(), crate::errors::AppError> {
    let Some(ubiquity_dir) = ubiquity_sync_subdir(folder) else {
        return Ok(());
    };
    if resolve_icloud_sync_file(&ubiquity_dir).is_some() {
        return Ok(());
    }
    let Some(legacy_dir) = legacy_clouddocs_sync_dir(folder) else {
        return Ok(());
    };
    if !legacy_dir.is_dir() {
        return Ok(());
    }
    let names: Vec<String> = std::fs::read_dir(&legacy_dir)
        .ok()
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let Some(legacy_path) = resolve_icloud_sync_file_from_names(&legacy_dir, &names) else {
        return Ok(());
    };
    let raw = std::fs::read(&legacy_path)?;
    if !raw.starts_with(b"SS2") {
        return Ok(());
    }
    let canonical = ubiquity_dir.join(crate::commands::sync::config::SYNC_FILENAME);
    coordinated_write_bytes(&canonical, &raw)?;
    eprintln!(
        "[subly][icloud] migrated CloudDocs → ubiquity from {} to {}",
        legacy_path.display(),
        canonical.display()
    );
    Ok(())
}
