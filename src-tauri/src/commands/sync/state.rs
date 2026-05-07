use super::{SyncConfig, SyncProviderType, SyncStatusDto};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRuntime {
    pub status: SyncStatusDto,
    pub oauth_pending_provider: Option<SyncProviderType>,
    /// PKCE `code_verifier` for the in-flight OAuth session (Google public client).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_pkce_verifier: Option<String>,
}

impl Default for SyncRuntime {
    fn default() -> Self {
        Self {
            oauth_pkce_verifier: None,
            status: SyncStatusDto {
                provider: None,
                enabled: false,
                last_synced: 0,
                syncing: false,
                error: None,
                remote_updated_at: 0,
                local_updated_at: 0,
                pending_update: false,
            },
            oauth_pending_provider: None,
        }
    }
}

pub fn sync_runtime() -> &'static Mutex<SyncRuntime> {
    static RUNTIME: OnceLock<Mutex<SyncRuntime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Mutex::new(SyncRuntime::default()))
}

pub fn sync_status_from_config(cfg: &SyncConfig) -> SyncStatusDto {
    SyncStatusDto {
        provider: cfg.provider.clone(),
        enabled: cfg.enabled,
        last_synced: cfg.last_synced,
        syncing: false,
        error: None,
        remote_updated_at: 0,
        local_updated_at: cfg.local_updated_at,
        pending_update: false,
    }
}
