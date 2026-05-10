use super::{
    clear_oauth_tokens, exchange_oauth_code, load_sync_config, now_ts, provider_access_token,
    provider_auth_url,
    provider_download, provider_upload, providers_list, save_oauth_tokens, save_sync_config,
    set_dropbox_app_secret, set_webdav_password,
    SyncEnableDto, SyncHasUpdateDto, SyncOkDto, SyncUiSchemaDto,
    sync_build_push_meta, sync_merge_with_tombstones, sync_payload_fits_limit, sync_push_revision_mismatch, sync_runtime,
    sync_should_pull, sync_status_from_config, token_key, SyncCredentialsDto, SyncPayload, SyncProviderType,
    KR_SYNC_DROPBOX_APP_SECRET, KR_SYNC_WEBDAV_PASSWORD, MAX_SYNC_PAYLOAD_BYTES,
};
use std::sync::OnceLock;
use tauri::Emitter;

fn persist_merged_snapshot(
    guard: &mut crate::state::AppStateInner,
    merged: &crate::models::AppDataDoc,
    app_config: &crate::models::AppConfigDoc,
) -> Result<(), String> {
    guard.apply_snapshot_typed_with_config(merged, app_config)
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
}

pub fn ensure_sync_scheduler_started(app: tauri::AppHandle) {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        tauri::async_runtime::spawn(async move {
            loop {
                if let Ok(cfg) = load_sync_config() {
                    if cfg.enabled {
                        if let Some(provider) = cfg.provider.clone() {
                            let token = match provider_access_token(&provider, &cfg).await {
                                Ok(v) => v,
                                Err(err) => {
                                    if let Ok(mut rt) = sync_runtime().lock() {
                                        rt.status.error = Some(err);
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
                tokio::time::sleep(std::time::Duration::from_secs(120)).await;
            }
        });
    });
}

pub async fn sync_dispatch_internal(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    command: SyncDispatchCommand,
) -> Result<SyncDispatchResponse, String> {
    let event = command.event_name().to_string();
    let mut cfg = load_sync_config()?;
    if matches!(command, SyncDispatchCommand::Init) {
        ensure_sync_scheduler_started(app.clone());
    }
    let current_status = {
        let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
        rt.status = sync_status_from_config(&cfg);
        rt.status.clone()
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
            // iCloud: do not block enable on folder discovery. On iOS the Drive path may appear
            // only after first sync/open in Files; push/pull will surface I/O errors if needed.
            cfg.provider = Some(provider.clone());
            cfg.enabled = true;
            save_sync_config(&cfg)?;
            {
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.oauth_pkce_verifier = None;
            }
            if let Some(url) = provider_auth_url(&provider, &cfg) {
                {
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                    rt.oauth_pending_provider = Some(provider);
                }
                SyncDispatchData::Enable(SyncEnableDto {
                    ok: true,
                    awaiting_oauth: Some(true),
                    auth_url: Some(url),
                    message_key: None,
                })
            } else {
                SyncDispatchData::Enable(SyncEnableDto {
                    ok: true,
                    awaiting_oauth: Some(false),
                    auth_url: None,
                    message_key: None,
                })
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
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.oauth_pending_provider = None;
                rt.oauth_pkce_verifier = None;
                rt.status = sync_status_from_config(&cfg);
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
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.oauth_pending_provider = None;
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
            let token = provider_access_token(&provider, &cfg).await?;
            let remote = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
            if let Some(payload) = remote {
                let remote_ts = payload.meta.updated_at.max(payload.meta.last_synced_at);
                {
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
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
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                    rt.status.pending_update = should_pull;
                }
                save_sync_config(&cfg)?;
                SyncDispatchData::HasUpdate(SyncHasUpdateDto {
                    has_update: should_pull,
                })
            } else {
                {
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
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
            let token = provider_access_token(&provider, &cfg).await?;
            if let Some(remote) = provider_download(&app, &provider, &cfg, token.as_deref()).await? {
                let (merged, merged_tombstones) = {
                    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
                    let local_ts = crate::state::load_deletion_tombstones(guard.db.as_ref())?;
                    let local = guard.doc()?;
                    drop(guard);
                    sync_merge_with_tombstones(local, remote.data, local_ts, remote.tombstones)?
                };
                {
                    let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
                    persist_merged_snapshot(&mut guard, &merged, &remote.app_config)?;
                    crate::state::replace_deletion_tombstones(guard.db.as_ref(), &merged_tombstones)?;
                }
                let remote_ts = remote.meta.updated_at.max(remote.meta.last_synced_at);
                cfg.local_updated_at = remote_ts;
                cfg.last_synced = now_ts();
                cfg.remote_revision = remote.meta.revision.unwrap_or_default();
                save_sync_config(&cfg)?;
                {
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                    rt.status = sync_status_from_config(&cfg);
                    rt.status.remote_updated_at = remote_ts;
                    rt.status.pending_update = false;
                }
                let _ = app.emit(
                    "app:data-changed",
                    serde_json::json!({ "entity": "appData", "action": "sync_pull" }),
                );
                #[cfg(target_os = "ios")]
                {
                    if let Ok(guard) = state.lock() {
                        crate::widget_snapshot::export_ios_widget_snapshot_from_guard(&guard);
                    }
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
            let token = provider_access_token(&provider, &cfg).await?;
            if !force_overwrite_remote {
                let remote = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
                if sync_push_revision_mismatch(&cfg.remote_revision, remote.as_ref()) {
                    return Ok(SyncDispatchResponse {
                        ok: true,
                        event,
                        data: SyncDispatchData::Ok(SyncOkDto::fail("sync_push_revision_conflict")),
                    });
                }
            }
            let (local_data, app_config, tombstones) = {
                let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
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
                return Err("sync payload too large".to_string());
            }
            provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await?;
            cfg.last_synced = payload.meta.last_synced_at;
            cfg.remote_revision = payload.meta.revision.clone().unwrap_or_default();
            cfg.local_updated_at = payload.meta.updated_at;
            save_sync_config(&cfg)?;
            {
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.status = sync_status_from_config(&cfg);
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
            let token = provider_access_token(&provider, &cfg).await?;
            let remote_opt = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
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
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
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
                    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
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
                    return Err("sync payload too large".to_string());
                }
                provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await?;
                cfg.last_synced = payload.meta.last_synced_at;
                cfg.remote_revision = payload.meta.revision.clone().unwrap_or_default();
                cfg.local_updated_at = payload.meta.updated_at;
                save_sync_config(&cfg)?;
                {
                    let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                    rt.status = sync_status_from_config(&cfg);
                }
                SyncDispatchData::Ok(SyncOkDto::success())
            }
        }
        SyncDispatchCommand::FlushBeforeExit { timeout_ms } => {
            let start = std::time::Instant::now();
            let result = if let Some(provider) = cfg.provider.clone() {
                let token = provider_access_token(&provider, &cfg).await?;
                let remote_opt = provider_download(&app, &provider, &cfg, token.as_deref()).await?;
                if sync_push_revision_mismatch(&cfg.remote_revision, remote_opt.as_ref()) {
                    false
                } else {
                    let (local_data, app_config, tombstones) = {
                        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
                        load_local_sync_snapshot(&guard)?
                    };
                    let meta = sync_build_push_meta(cfg.local_updated_at, cfg.device_id.clone())?;
                    let payload = SyncPayload {
                        data: local_data,
                        app_config,
                        meta,
                        tombstones,
                    };
                    provider_upload(&app, &provider, &cfg, &payload, token.as_deref()).await.is_ok()
                }
            } else {
                true
            };
            let timed_out = start.elapsed().as_millis() > timeout_ms as u128;
            SyncDispatchData::Ok(SyncOkDto {
                ok: result && !timed_out,
                pending_update: None,
                message_key: None,
            })
        }
        SyncDispatchCommand::DismissPending => {
            {
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.status.pending_update = false;
            }
            SyncDispatchData::Ok(SyncOkDto::success())
        }
    };

    Ok(SyncDispatchResponse { ok: true, event, data })
}
