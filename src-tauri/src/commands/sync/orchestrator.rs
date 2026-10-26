use super::{
    clear_oauth_tokens, exchange_oauth_code, load_sync_config, now_ts, provider_access_token,
    provider_auth_url,
    provider_download, provider_upload, providers_list, save_oauth_tokens, save_sync_config,
    set_dropbox_app_secret, set_webdav_password,
    SyncEnableDto, SyncHasUpdateDto, SyncOkDto, SyncUiSchemaDto,
    sync_build_push_meta, sync_merge_with_tombstones, sync_payload_fits_limit, sync_push_revision_mismatch, sync_runtime,
    sync_should_pull, sync_oauth_message_key, oauth_credentials_missing_key, oauth_connect_ready,
    refresh_runtime_status, webdav_credentials_ready, token_key, SyncCredentialsDto, SyncPayload, SyncProviderType,
    KR_SYNC_DROPBOX_APP_SECRET, KR_SYNC_WEBDAV_PASSWORD, MAX_SYNC_PAYLOAD_BYTES,
};
use std::sync::OnceLock;
use tauri::Emitter;
use tokio::sync::Mutex;

/// Serializes network + merge + Redb sync work (IPC and background poll).
static GLOBAL_SYNC_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn sync_lock() -> &'static Mutex<()> {
    GLOBAL_SYNC_LOCK.get_or_init(|| Mutex::new(()))
}

fn icloud_fail_response(
    event: &str,
    err: crate::errors::AppError,
) -> Result<SyncDispatchResponse, crate::errors::AppError> {
    if let Some(key) = provider_io_message_key(&err) {
        return Ok(SyncDispatchResponse {
            ok: true,
            event: event.to_string(),
            data: SyncDispatchData::Ok(SyncOkDto::fail(key)),
        });
    }
    Err(err)
}

fn oauth_token_fail_response(
    event: &str,
    err: crate::errors::AppError,
) -> Result<SyncDispatchResponse, crate::errors::AppError> {
    if let Some(key) = sync_oauth_message_key(&err) {
        return Ok(SyncDispatchResponse {
            ok: true,
            event: event.to_string(),
            data: SyncDispatchData::Ok(SyncOkDto::fail(key)),
        });
    }
    Err(err)
}

fn provider_io_message_key(err: &crate::errors::AppError) -> Option<&'static str> {
    super::providers::provider_io_message_key(err).or_else(|| sync_oauth_message_key(err))
}

fn persist_merged_snapshot(
    guard: &mut crate::state::AppStateInner,
    merged: &crate::models::AppDataDoc,
    app_config: &crate::models::AppConfigDoc,
) -> Result<(), crate::errors::AppError> {
    guard.apply_merged_snapshot_with_config(merged, app_config)
}

fn load_local_sync_snapshot(
    guard: &crate::state::AppStateInner,
) -> Result<
    (
        crate::models::AppDataDoc,
        crate::models::AppConfigDoc,
        Vec<crate::models::DeletionTombstone>,
    ),
    String,
> {
    use crate::state::{read_singleton_bin_typed, T2_CONFIG};
    Ok((
        guard.doc()?,
        read_singleton_bin_typed(guard.db.as_ref(), T2_CONFIG, crate::models::AppConfigDoc::default())?,
        crate::state::load_deletion_tombstones(guard.db.as_ref())?,
    ))
}

#[derive(Debug, Clone)]
pub enum SyncDispatchCommand {
    Init,
    GetStatus,
    GetConfig,
    SetCredentials {
        provider: SyncProviderType,
        credentials: SyncCredentialsDto,
    },
    Enable {
        provider: SyncProviderType,
    },
    Disable,
    OauthFinish {
        code: String,
        provider: Option<SyncProviderType>,
    },
    CheckRemote,
    PullRemote,
    PushLocal { force_overwrite_remote: bool },
    SyncNow,
    FlushBeforeExit {
        timeout_ms: u64,
    },
    DismissPending,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SyncDispatchData {
    UiSchema(SyncUiSchemaDto),
    Status(super::SyncStatusDto),
    Config(super::SyncConfig),
    Enable(SyncEnableDto),
    HasUpdate(SyncHasUpdateDto),
    Ok(SyncOkDto),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDispatchResponse {
    pub ok: bool,
    pub event: String,
    pub data: SyncDispatchData,
}

impl SyncDispatchCommand {
    fn event_name(&self) -> &'static str {
        match self {
            SyncDispatchCommand::Init => "init",
            SyncDispatchCommand::GetStatus => "get_status",
            SyncDispatchCommand::GetConfig => "get_config",
            SyncDispatchCommand::SetCredentials { .. } => "set_credentials",
            SyncDispatchCommand::Enable { .. } => "enable",
            SyncDispatchCommand::Disable => "disable",
            SyncDispatchCommand::OauthFinish { .. } => "oauth_finish",
            SyncDispatchCommand::CheckRemote => "check_remote",
            SyncDispatchCommand::PullRemote => "pull_remote",
            SyncDispatchCommand::PushLocal { .. } => "push_local",
            SyncDispatchCommand::SyncNow => "sync_now",
            SyncDispatchCommand::FlushBeforeExit { .. } => "flush_before_exit",
            SyncDispatchCommand::DismissPending => "dismiss_pending",
        }
    }

    /// Remote I/O and snapshot merge/push must not overlap (scheduler or UI).
    fn needs_sync_lock(&self) -> bool {
        matches!(
            self,
            SyncDispatchCommand::CheckRemote
                | SyncDispatchCommand::PullRemote
                | SyncDispatchCommand::PushLocal { .. }
                | SyncDispatchCommand::SyncNow
                | SyncDispatchCommand::FlushBeforeExit { .. }
        )
    }
}

const SYNC_SCHEDULER_TICK: std::time::Duration = std::time::Duration::from_secs(120);

pub fn ensure_sync_scheduler_started(app: tauri::AppHandle) {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        tauri::async_runtime::spawn(async move {
            loop {
                if let Ok(cfg) = load_sync_config() {
                    if cfg.enabled {
                        // Busy (manual sync holds the lock): wait out the tick. A bare `continue`
                        // skipped the sleep below and spun a runtime worker at 100% CPU.
                        let Ok(_guard) = sync_lock().try_lock() else {
                            tokio::time::sleep(SYNC_SCHEDULER_TICK).await;
                            continue;
                        };
                        if let Some(provider) = cfg.provider.clone() {
                            let token = match provider_access_token(&provider, &cfg).await {
                                Ok(v) => v,
                                Err(err) => {
                                    if let Ok(mut rt) = sync_runtime().lock() {
                                        rt.status.error = Some(err.to_string());
                                    }
                                    None
                                }
                            };
                            if let Ok(remote) = provider_download(&app, &provider, &cfg, token.as_deref()).await {
                                if let Ok(mut rt) = sync_runtime().lock() {
                                    if let Some(payload) = remote {
                                        let remote_ts = payload.meta.updated_at.max(payload.meta.last_synced_at);
                                        rt.status.remote_updated_at = remote_ts;
                                        rt.status.pending_update = sync_should_pull(
                                            remote_ts,
                                            payload.meta.device_id,
                                            cfg.local_updated_at,
                                            cfg.device_id.clone(),
                                        )
                                        .unwrap_or(false);
                                    } else {
                                        rt.status.pending_update = false;
                                    }
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(SYNC_SCHEDULER_TICK).await;
            }
        });
    });
}

pub async fn sync_dispatch_internal(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    command: SyncDispatchCommand,
) -> Result<SyncDispatchResponse, crate::errors::AppError> {
    let event = command.event_name().to_string();
    let mut cfg = load_sync_config()?;
    if matches!(command, SyncDispatchCommand::Init) {
        ensure_sync_scheduler_started(app.clone());
    }
    let current_status = {
        let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
        refresh_runtime_status(&cfg, &mut rt)
    };

    let _sync_guard = if command.needs_sync_lock() {
        Some(sync_lock().lock().await)
    } else {
        None
    };

    let data = match command {
        SyncDispatchCommand::Init => SyncDispatchData::UiSchema(SyncUiSchemaDto {
            status: current_status,
            config: cfg,
            providers: providers_list(),
        }),
        SyncDispatchCommand::GetStatus => SyncDispatchData::Status(current_status),
        SyncDispatchCommand::GetConfig => SyncDispatchData::Config(cfg),
        SyncDispatchCommand::SetCredentials { provider, credentials } => {
            match provider {
                SyncProviderType::Gdrive => {}
                SyncProviderType::Dropbox => {
                    cfg.dropbox_app_key = credentials.app_key.unwrap_or_default();
                    if let Some(s) = credentials.app_secret {
                        set_dropbox_app_secret(&s)?;
                    }
                }
                SyncProviderType::Onedrive => {
                    cfg.onedrive_client_id = credentials.client_id.unwrap_or_default();
                }
                SyncProviderType::Webdav => {
                    cfg.webdav_url = credentials.server_url.unwrap_or_default();
                    cfg.webdav_username = credentials.username.unwrap_or_default();
                    if let Some(s) = credentials.password {
                        set_webdav_password(&s)?;
                    }
                }
                SyncProviderType::Icloud => {}
            }
            save_sync_config(&cfg)?;
            SyncDispatchData::Ok(SyncOkDto::success())
        }
        SyncDispatchCommand::Enable { provider } => {
            cfg.provider = Some(provider.clone());
            cfg.enabled = false;
            {
                let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                rt.oauth_pkce_verifier = None;
                rt.oauth_pending_provider = None;
            }

            match &provider {
                SyncProviderType::Gdrive | SyncProviderType::Dropbox | SyncProviderType::Onedrive => {
                    if !oauth_connect_ready(&provider, &cfg) {
                        cfg.provider = None;
                        save_sync_config(&cfg)?;
                        SyncDispatchData::Enable(SyncEnableDto {
                            ok: false,
                            awaiting_oauth: None,
                            auth_url: None,
                            message_key: Some(oauth_credentials_missing_key(&provider).to_string()),
                        })
                    } else if let Some(url) = provider_auth_url(&provider, &cfg) {
                        save_sync_config(&cfg)?;
                        {
                            let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                            rt.oauth_pending_provider = Some(provider.clone());
                            refresh_runtime_status(&cfg, &mut rt);
                        }
                        SyncDispatchData::Enable(SyncEnableDto {
                            ok: true,
                            awaiting_oauth: Some(true),
                            auth_url: Some(url),
                            message_key: None,
                        })
                    } else {
                        cfg.provider = None;
                        save_sync_config(&cfg)?;
                        SyncDispatchData::Enable(SyncEnableDto {
                            ok: false,
                            awaiting_oauth: None,
                            auth_url: None,
                            message_key: Some(oauth_credentials_missing_key(&provider).to_string()),
                        })
                    }
                }
                SyncProviderType::Webdav => {
                    if !webdav_credentials_ready(&cfg) {
                        cfg.provider = None;
                        save_sync_config(&cfg)?;
                        SyncDispatchData::Enable(SyncEnableDto {
                            ok: false,
                            awaiting_oauth: None,
                            auth_url: None,
                            message_key: Some("sync_webdav_credentials_missing".to_string()),
                        })
                    } else {
                        cfg.enabled = true;
                        save_sync_config(&cfg)?;
                        {
                            let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                            refresh_runtime_status(&cfg, &mut rt);
                        }
                        SyncDispatchData::Enable(SyncEnableDto {
                            ok: true,
                            awaiting_oauth: Some(false),
                            auth_url: None,
                            message_key: None,
                        })
                    }
                }
                SyncProviderType::Icloud => {
                    cfg.enabled = true;
                    save_sync_config(&cfg)?;
                    {
                        let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                        refresh_runtime_status(&cfg, &mut rt);
                    }
                    SyncDispatchData::Enable(SyncEnableDto {
                        ok: true,
                        awaiting_oauth: Some(false),
                        auth_url: None,
                        message_key: None,
                    })
                }
            }
        }
        SyncDispatchCommand::Disable => {
            if let Some(provider) = cfg.provider.clone() {
                if let Some(k) = token_key(&provider) {
                    clear_oauth_tokens(k)?;
                }
            }
            let _ = crate::keyring_store::delete(KR_SYNC_DROPBOX_APP_SECRET);
            let _ = crate::keyring_store::delete(KR_SYNC_WEBDAV_PASSWORD);
            cfg.provider = None;
            cfg.enabled = false;
            cfg.remote_revision.clear();
            save_sync_config(&cfg)?;
            {
                let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                rt.oauth_pending_provider = None;
                rt.oauth_pkce_verifier = None;
                refresh_runtime_status(&cfg, &mut rt);
            }
            SyncDispatchData::Ok(SyncOkDto::success())
        }
        SyncDispatchCommand::OauthFinish { code, provider } => {
            let provider = provider
                .or_else(|| cfg.provider.clone())
                .ok_or("provider missing for oauth_finish")?;
            let tokens = exchange_oauth_code(&provider, &cfg, &code).await?;
            let key = token_key(&provider).ok_or("provider does not support oauth")?;
            save_oauth_tokens(key, &tokens)?;
            cfg.enabled = true;
            cfg.provider = Some(provider.clone());
            save_sync_config(&cfg)?;
            {
                let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                rt.oauth_pending_provider = None;
                refresh_runtime_status(&cfg, &mut rt);
            }
            SyncDispatchData::Ok(SyncOkDto::success())
        }
        SyncDispatchCommand::CheckRemote => {
            let provider = match cfg.provider.clone() {
                Some(p) => p,
                None => {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::HasUpdate(SyncHasUpdateDto { has_update: false }),
                    })
                }
            };
            let token = match provider_access_token(&provider, &cfg).await {
                Ok(t) => t,
                Err(e) => return oauth_token_fail_response(&event, e),
            };
            let remote = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
            if let Some(payload) = remote {
                let remote_ts = payload.meta.updated_at.max(payload.meta.last_synced_at);
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    rt.status.remote_updated_at = remote_ts;
                }
                cfg.remote_revision = payload.meta.revision.unwrap_or_default();
                let should_pull = sync_should_pull(
                    remote_ts,
                    payload.meta.device_id,
                    cfg.local_updated_at,
                    cfg.device_id.clone(),
                )?;
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    rt.status.pending_update = should_pull;
                }
                save_sync_config(&cfg)?;
                SyncDispatchData::HasUpdate(SyncHasUpdateDto {
                    has_update: should_pull,
                })
            } else {
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    rt.status.pending_update = false;
                }
                SyncDispatchData::HasUpdate(SyncHasUpdateDto { has_update: false })
            }
        }
        SyncDispatchCommand::PullRemote => {
            let provider = match cfg.provider.clone() {
                Some(p) => p,
                None => {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::Ok(SyncOkDto::fail("sync_no_provider")),
                    })
                }
            };
            let token = match provider_access_token(&provider, &cfg).await {
                Ok(t) => t,
                Err(e) => return oauth_token_fail_response(&event, e),
            };
            let remote = match provider_download(&app, &provider, &cfg, token.as_deref()).await {
                Ok(r) => r,
                Err(e) => return icloud_fail_response(&event, e),
            };
            if let Some(remote) = remote {
                let (merged, merged_tombstones) = {
                    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
                    let local_ts = crate::state::load_deletion_tombstones(guard.db.as_ref())?;
                    let local = guard.doc()?;
                    drop(guard);
                    sync_merge_with_tombstones(local, remote.data, local_ts, remote.tombstones)?
                };
                {
                    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
                    persist_merged_snapshot(&mut guard, &merged, &remote.app_config)?;
                    crate::state::replace_deletion_tombstones(guard.db.as_ref(), &merged_tombstones)?;
                    crate::state::prune_deletion_tombstones_older_than(
                        guard.db.as_ref(),
                        crate::state::DELETION_TOMBSTONE_RETAIN_MS,
                    )?;
                }
                let remote_ts = remote.meta.updated_at.max(remote.meta.last_synced_at);
                cfg.local_updated_at = remote_ts;
                cfg.last_synced = now_ts();
                cfg.remote_revision = remote.meta.revision.unwrap_or_default();
                save_sync_config(&cfg)?;
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    refresh_runtime_status(&cfg, &mut rt);
                    rt.status.remote_updated_at = remote_ts;
                    rt.status.pending_update = false;
                }
                let _ = app.emit(
                    "app:data-changed",
                    serde_json::json!({ "entity": "appData", "action": "sync_pull" }),
                );
                #[cfg(any(target_os = "ios", target_os = "android"))]
                {
                    crate::widget_snapshot::export_widget_snapshot_from_app(&app);
                    crate::commands::notifications::reschedule_from_app(&app);
                }
                SyncDispatchData::Ok(SyncOkDto::success())
            } else {
                SyncDispatchData::Ok(SyncOkDto::fail("sync_pull_no_remote_file"))
            }
        }
        SyncDispatchCommand::PushLocal {
            force_overwrite_remote,
        } => {
            let provider = match cfg.provider.clone() {
                Some(p) => p,
                None => {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::Ok(SyncOkDto::fail("sync_no_provider")),
                    })
                }
            };
            let token = match provider_access_token(&provider, &cfg).await {
                Ok(t) => t,
                Err(e) => return oauth_token_fail_response(&event, e),
            };
            if !force_overwrite_remote {
                let remote = match provider_download(&app, &provider, &cfg, token.as_deref()).await {
                    Ok(r) => r,
                    Err(e) => return icloud_fail_response(&event, e),
                };
                if sync_push_revision_mismatch(&cfg.remote_revision, remote.as_ref()) {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::Ok(SyncOkDto::fail("sync_push_revision_conflict")),
                    });
                }
            }
            let (local_data, app_config, tombstones) = {
                let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
                load_local_sync_snapshot(&guard)?
            };
            let meta = sync_build_push_meta(cfg.local_updated_at, cfg.device_id.clone())?;
            let payload = SyncPayload {
                data: local_data,
                app_config,
                meta,
                tombstones,
            };
            if !sync_payload_fits_limit(payload.clone(), MAX_SYNC_PAYLOAD_BYTES)? {
                return Err(crate::errors::AppError::from("sync payload too large"));
            }
            provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await?;
            cfg.last_synced = payload.meta.last_synced_at;
            cfg.remote_revision = payload.meta.revision.clone().unwrap_or_default();
            cfg.local_updated_at = payload.meta.updated_at;
            save_sync_config(&cfg)?;
            {
                let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                refresh_runtime_status(&cfg, &mut rt);
                rt.status.remote_updated_at = payload.meta.updated_at;
            }
            SyncDispatchData::Ok(SyncOkDto::success())
        }
        SyncDispatchCommand::SyncNow => {
            let provider = match cfg.provider.clone() {
                Some(p) => p,
                None => {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::Ok(SyncOkDto::fail("sync_no_provider")),
                    })
                }
            };
            let token = match provider_access_token(&provider, &cfg).await {
                Ok(t) => t,
                Err(e) => return oauth_token_fail_response(&event, e),
            };
            let remote_opt = match provider_download(&app, &provider, &cfg, token.as_deref()).await {
                Ok(r) => r,
                Err(e) => return icloud_fail_response(&event, e),
            };
            let has_update = if let Some(ref remote) = remote_opt {
                let remote_ts = remote.meta.updated_at.max(remote.meta.last_synced_at);
                sync_should_pull(
                    remote_ts,
                    remote.meta.device_id.clone(),
                    cfg.local_updated_at,
                    cfg.device_id.clone(),
                )?
            } else {
                false
            };
            if has_update {
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    rt.status.pending_update = true;
                }
                SyncDispatchData::Ok(SyncOkDto {
                    ok: false,
                    pending_update: Some(true),
                    message_key: None,
                })
            } else if sync_push_revision_mismatch(&cfg.remote_revision, remote_opt.as_ref()) {
                SyncDispatchData::Ok(SyncOkDto::fail("sync_push_revision_conflict"))
            } else {
                let (local_data, app_config, tombstones) = {
                    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
                    load_local_sync_snapshot(&guard)?
                };
                let meta = sync_build_push_meta(cfg.local_updated_at, cfg.device_id.clone())?;
                let payload = SyncPayload {
                    data: local_data,
                    app_config,
                    meta,
                    tombstones,
                };
                if !sync_payload_fits_limit(payload.clone(), MAX_SYNC_PAYLOAD_BYTES)? {
                    return Err(crate::errors::AppError::from("sync payload too large"));
                }
                provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await?;
                cfg.last_synced = payload.meta.last_synced_at;
                cfg.remote_revision = payload.meta.revision.clone().unwrap_or_default();
                cfg.local_updated_at = payload.meta.updated_at;
                save_sync_config(&cfg)?;
                {
                    let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                    refresh_runtime_status(&cfg, &mut rt);
                }
                SyncDispatchData::Ok(SyncOkDto::success())
            }
        }
        SyncDispatchCommand::FlushBeforeExit { timeout_ms } => {
            // Runs while the window is closing: bounded by `timeout_ms` for real (it used to be
            // measured only after the fact), and any error just means "not flushed" so the window
            // still closes.
            let result = if let Some(provider) = cfg.provider.clone() {
                let flush = async {
                    let token = provider_access_token(&provider, &cfg).await?;
                    let remote_opt = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
                    if sync_push_revision_mismatch(&cfg.remote_revision, remote_opt.as_ref()) {
                        return Ok::<bool, crate::errors::AppError>(false);
                    }
                    let (local_data, app_config, tombstones) = {
                        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
                        load_local_sync_snapshot(&guard)?
                    };
                    let meta = sync_build_push_meta(cfg.local_updated_at, cfg.device_id.clone())?;
                    let payload = SyncPayload {
                        data: local_data,
                        app_config,
                        meta,
                        tombstones,
                    };
                    if !sync_payload_fits_limit(payload.clone(), MAX_SYNC_PAYLOAD_BYTES)? {
                        return Ok(false);
                    }
                    provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await?;
                    Ok(true)
                };
                match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), flush).await {
                    Ok(Ok(flushed)) => flushed,
                    Ok(Err(e)) => {
                        eprintln!("[subly][sync] flush before exit failed: {}", e);
                        false
                    }
                    Err(_) => false,
                }
            } else {
                true
            };
            SyncDispatchData::Ok(SyncOkDto {
                ok: result,
                pending_update: None,
                message_key: None,
            })
        }
        SyncDispatchCommand::DismissPending => {
            {
                let mut rt = sync_runtime().lock().map_err(|_| crate::errors::AppError::SyncRuntimeLockPoisoned)?;
                rt.status.pending_update = false;
            }
            SyncDispatchData::Ok(SyncOkDto::success())
        }
    };

    Ok(SyncDispatchResponse { ok: true, event, data })
}
