//! Флаг «идёт тяжёлая запись снимка / tombstones в Redb» — фоновый poll синка ждёт.
use std::sync::atomic::{AtomicBool, Ordering};

static SYNC_REDB_WRITE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) struct SyncRedbWriteGuard;

impl Drop for SyncRedbWriteGuard {
    fn drop(&mut self) {
        SYNC_REDB_WRITE_ACTIVE.store(false, Ordering::Release);
    }
}

/// Установить флаг до выхода из области видимости [`SyncRedbWriteGuard`].
pub(crate) fn sync_redb_write_begin() -> SyncRedbWriteGuard {
    SYNC_REDB_WRITE_ACTIVE.store(true, Ordering::Release);
    SyncRedbWriteGuard
}

pub(crate) fn sync_redb_write_is_active() -> bool {
    SYNC_REDB_WRITE_ACTIVE.load(Ordering::Acquire)
}
