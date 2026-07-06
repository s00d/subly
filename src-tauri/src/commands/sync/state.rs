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
            status: sync_status_snapshot(&SyncConfig::default(), None),
            oauth_pending_provider: None,
        }
    }
}

pub fn sync_runtime() -> &'static Mutex<SyncRuntime> {
    static RUNTIME: OnceLock<Mutex<SyncRuntime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Mutex::new(SyncRuntime::default()))
}

pub fn sync_status_snapshot(
    cfg: &SyncConfig,
    oauth_pending_provider: Option<&SyncProviderType>,
) -> SyncStatusDto {
    SyncStatusDto {
        provider: cfg.provider.clone(),
        enabled: cfg.enabled,
        last_synced: cfg.last_synced,
        syncing: false,
        error: None,
        remote_updated_at: 0,
        local_updated_at: cfg.local_updated_at,
        pending_update: false,
        oauth_pending: oauth_pending_provider.is_some(),
        oauth_pending_provider: oauth_pending_provider.cloned(),
    }
}

pub fn sync_status_from_config(cfg: &SyncConfig) -> SyncStatusDto {
    sync_status_snapshot(cfg, None)
}

/// Rebuild status from config + runtime, preserving in-flight UI fields.
pub fn refresh_runtime_status(cfg: &SyncConfig, rt: &mut SyncRuntime) -> SyncStatusDto {
    let pending = rt.oauth_pending_provider.as_ref();
    let mut next = sync_status_snapshot(cfg, pending);
    next.pending_update = rt.status.pending_update;
    next.remote_updated_at = rt.status.remote_updated_at;
    next.error = rt.status.error.clone();
    next.syncing = rt.status.syncing;
    rt.status = next.clone();
    next
}

pub(crate) fn webdav_credentials_ready(cfg: &SyncConfig) -> bool {
    !cfg.webdav_url.trim().is_empty() && !cfg.webdav_username.trim().is_empty()
}

pub(crate) fn dropbox_credentials_ready(cfg: &SyncConfig) -> bool {
    !cfg.dropbox_app_key.trim().is_empty()
        && crate::keyring_store::exists_for_background(super::config::KR_SYNC_DROPBOX_APP_SECRET)
            .unwrap_or(false)
}

pub(crate) fn onedrive_credentials_ready(cfg: &SyncConfig) -> bool {
    !cfg.onedrive_client_id.trim().is_empty()
}

fn oauth_provider_credentials_ready(provider: &SyncProviderType, cfg: &SyncConfig) -> bool {
    match provider {
        SyncProviderType::Gdrive => true,
        SyncProviderType::Dropbox => dropbox_credentials_ready(cfg),
        SyncProviderType::Onedrive => onedrive_credentials_ready(cfg),
        _ => false,
    }
}

pub(crate) fn oauth_credentials_missing_key(provider: &SyncProviderType) -> &'static str {
    match provider {
        SyncProviderType::Dropbox => "sync_dropbox_credentials_missing",
        SyncProviderType::Onedrive => "sync_onedrive_credentials_missing",
        _ => "sync_oauth_required",
    }
}

/// Whether an OAuth provider can start the browser sign-in flow.
pub(crate) fn oauth_connect_ready(provider: &SyncProviderType, cfg: &SyncConfig) -> bool {
    oauth_provider_credentials_ready(provider, cfg)
}
