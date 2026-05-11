//! iCloud access on Apple platforms via **ubiquity container** (`NSFileManager::URLForUbiquityContainerIdentifier`)
//! and coordinated I/O (`NSFileCoordinator`). Avoids hard-coded `com~apple~CloudDocs` paths on iOS and
//! matches App Store / sandbox expectations.

use core::ptr::NonNull;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use block2::RcBlock;
use objc2::rc::autoreleasepool;
use objc2_foundation::{
    NSFileCoordinator, NSFileCoordinatorReadingOptions, NSFileCoordinatorWritingOptions, NSFileManager, NSError,
    NSString, NSURL,
};

#[cfg(any(target_os = "ios", target_os = "macos"))]
const APP_ICLOUD_CONTAINER_ID: &str = "iCloud.com.s00d.subly";

fn path_to_file_url(path: &Path) -> Result<objc2::rc::Retained<NSURL>, crate::errors::AppError> {
    let s = path
        .to_str()
        .ok_or_else(|| crate::errors::AppError::from("non-utf8 path"))?;
    Ok(NSURL::fileURLWithPath(&NSString::from_str(s)))
}

/// `Documents/<relative_folder>` inside the app’s iCloud container (see entitlements).
pub(crate) fn ubiquity_sync_subdir(relative_folder: &str) -> Option<PathBuf> {
    autoreleasepool(|_pool| {
        let fm = NSFileManager::defaultManager();
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        let container = fm.URLForUbiquityContainerIdentifier(Some(&NSString::from_str(APP_ICLOUD_CONTAINER_ID)))?;
        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        let container = fm.URLForUbiquityContainerIdentifier(None)?;
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(cpath) = container.path() {
            #[cfg(target_os = "ios")]
            eprintln!("[subly][icloud][ios] container={}", cpath);
            #[cfg(target_os = "macos")]
            eprintln!("[subly][icloud][macos] container={}", cpath);
        }
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

enum LocalItemState {
    Ready,
    Missing,
}

fn prepare_ubiquitous_item_local(path: &Path) -> Result<LocalItemState, crate::errors::AppError> {
    autoreleasepool(|_pool| {
        let url = path_to_file_url(path)?;
        let fm = NSFileManager::defaultManager();
        let is_ubiquitous = fm.isUbiquitousItemAtURL(&url);
        if path.exists() {
            return Ok(LocalItemState::Ready);
        }
        if !is_ubiquitous {
            return Ok(LocalItemState::Missing);
        }
        let dl_res = fm.startDownloadingUbiquitousItemAtURL_error(&url);
        for _ in 0..12 {
            if path.exists() {
                return Ok(LocalItemState::Ready);
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        match dl_res {
            Ok(()) => Err(crate::errors::AppError::from(format!(
                "icloud file is not materialized yet (ubiquitous={}, path={})",
                is_ubiquitous,
                path.display()
            ))),
            Err(e) => Err(crate::errors::AppError::from(format!(
                "startDownloadingUbiquitousItem failed (ubiquitous={}, path={}, err={})",
                is_ubiquitous,
                path.display(),
                e.localizedDescription()
            ))),
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
            &*block,
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
            &*block,
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
            std::fs::create_dir_all(parent)?;
        }
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
            let res = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&pb)
                .and_then(|mut f| std::io::Write::write_all(&mut f, &data).map(|_| ()));
            *out_cb.lock().expect("lock") = Some(res.map_err(crate::errors::AppError::from));
        });
        let mut coord_err: Option<objc2::rc::Retained<NSError>> = None;
        coordinator.coordinateWritingItemAtURL_options_error_byAccessor(
            &url,
            NSFileCoordinatorWritingOptions::ForReplacing,
            Some(&mut coord_err),
            &*block,
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
