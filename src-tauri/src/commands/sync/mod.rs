mod config;
pub(crate) mod oauth;
mod orchestrator;
pub(crate) mod providers;
mod state;
mod types;
mod wire;

use serde::de::DeserializeOwned;
pub use types::{SyncProviderType, SyncStatusDto};

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub(crate) use config::{
    clear_oauth_tokens, load_oauth_tokens, load_sync_config, save_oauth_tokens, save_sync_config,
    set_dropbox_app_secret, set_webdav_password, token_key, webdav_password, KR_SYNC_DROPBOX_APP_SECRET,
    KR_SYNC_WEBDAV_PASSWORD, MAX_SYNC_PAYLOAD_BYTES, OAUTH_REDIRECT_URI, SYNC_FILENAME, SYNC_ICLOUD_FOLDER,
};
#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub(crate) use config::{
    clear_oauth_tokens, load_oauth_tokens, load_sync_config, save_oauth_tokens, save_sync_config,
    set_dropbox_app_secret, set_webdav_password, token_key, webdav_password, KR_SYNC_DROPBOX_APP_SECRET,
    KR_SYNC_WEBDAV_PASSWORD, MAX_SYNC_PAYLOAD_BYTES, OAUTH_REDIRECT_URI, SYNC_FILENAME,
};
pub(crate) use wire::{decode_sync_payload, encode_sync_payload};
pub(crate) use oauth::{exchange_oauth_code, provider_access_token};
pub(crate) use orchestrator::{sync_dispatch_internal, SyncDispatchCommand, SyncDispatchResponse};
pub(crate) use providers::{provider_auth_url, provider_download, provider_upload, providers_list};
pub(crate) use state::{sync_runtime, sync_status_from_config};
pub(crate) use types::{OAuthTokens, SyncConfig, SyncMeta, SyncPayload};
use crate::models::{AppDataDoc, DeletionTombstone, TombstoneEntityKind};
use self::providers::ProviderInfo;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncUiSchemaDto {
    pub status: SyncStatusDto,
    pub config: SyncConfig,
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOkDto {
    pub ok: bool,
    pub pending_update: Option<bool>,
    /// i18n key for the frontend (`sync_pull_no_remote_file`, `sync_no_provider`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_key: Option<String>,
}

impl SyncOkDto {
    pub(crate) fn success() -> Self {
        Self {
            ok: true,
            pending_update: None,
            message_key: None,
        }
    }

    pub(crate) fn fail(message_key: impl Into<String>) -> Self {
        Self {
            ok: false,
            pending_update: None,
            message_key: Some(message_key.into()),
        }
    }

}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEnableDto {
    pub ok: bool,
    pub awaiting_oauth: Option<bool>,
    pub auth_url: Option<String>,
    /// i18n key when `ok` is false (e.g. `sync_icloud_unavailable`).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_key: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHasUpdateDto {
    pub has_update: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncCredentialsDto {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
    pub server_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

fn extract_data<T: DeserializeOwned>(response: SyncDispatchResponse) -> Result<T, String> {
    let data = serde_json::to_value(response.data).map_err(|e| e.to_string())?;
    serde_json::from_value(data).map_err(|e| e.to_string())
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn generate_device_id() -> String {
    format!("dev_{}", now_ts())
}


#[tauri::command]
pub fn sync_should_pull(
    remote_updated_at: i64,
    remote_device_id: String,
    local_updated_at: i64,
    local_device_id: String,
) -> Result<bool, String> {
    Ok(remote_updated_at > local_updated_at && remote_device_id != local_device_id)
}

#[tauri::command]
pub fn sync_has_push_conflict(
    remote_updated_at: i64,
    remote_device_id: String,
    local_updated_at: i64,
    local_device_id: String,
    local_remote_revision: String,
    remote_revision: String,
) -> Result<bool, String> {
    Ok(
        remote_device_id != local_device_id
            && remote_updated_at > local_updated_at
            && !local_remote_revision.is_empty()
            && remote_revision != local_remote_revision,
    )
}

#[tauri::command]
pub fn sync_merge_app_data(local: AppDataDoc, remote: AppDataDoc) -> Result<AppDataDoc, String> {
    Ok(merge_app_data_rows(local, remote))
}

fn merge_app_data_rows(local: AppDataDoc, remote: AppDataDoc) -> AppDataDoc {
    AppDataDoc {
        subscriptions: merge_timestamped_by_id(local.subscriptions, remote.subscriptions, |x| x.id.clone(), |x| x.updated_at),
        expenses: merge_timestamped_by_id(local.expenses, remote.expenses, |x| x.id.clone(), |x| x.updated_at),
        categories: merge_by_id_prefer_remote(local.categories, remote.categories, |x| x.id.clone()),
        currencies: merge_by_id_prefer_remote(local.currencies, remote.currencies, |x| x.id.clone()),
        household: merge_by_id_prefer_remote(local.household, remote.household, |x| x.id.clone()),
        payment_methods: merge_by_id_prefer_remote(local.payment_methods, remote.payment_methods, |x| x.id.clone()),
        tags: merge_by_id_prefer_remote(local.tags, remote.tags, |x| x.id.clone()),
        settings: remote.settings,
    }
}

fn merge_tombstone_vecs(local: Vec<DeletionTombstone>, remote: Vec<DeletionTombstone>) -> Vec<DeletionTombstone> {
    let mut m: HashMap<(TombstoneEntityKind, String), DeletionTombstone> = HashMap::new();
    for t in local.into_iter().chain(remote) {
        let k = (t.entity_kind, t.entity_id.clone());
        match m.get_mut(&k) {
            Some(existing) => {
                if t.deleted_at > existing.deleted_at {
                    *existing = t;
                }
            }
            None => {
                m.insert(k, t);
            }
        }
    }
    m.into_values().collect()
}

/// Apply merged tombstones: timestamped rows survive only if `updated_at` is newer than the deletion.
/// Catalog rows are removed unconditionally when a tombstone exists (delete wins over merged row).
fn apply_tombstone_filter(doc: &mut AppDataDoc, ts: &[DeletionTombstone]) {
    for t in ts {
        match t.entity_kind {
            TombstoneEntityKind::Subscription => {
                doc.subscriptions
                    .retain(|x| x.id != t.entity_id || x.updated_at > t.deleted_at);
            }
            TombstoneEntityKind::Expense => {
                doc.expenses.retain(|x| x.id != t.entity_id || x.updated_at > t.deleted_at);
            }
            TombstoneEntityKind::Category => doc.categories.retain(|x| x.id != t.entity_id),
            TombstoneEntityKind::Currency => doc.currencies.retain(|x| x.id != t.entity_id),
            TombstoneEntityKind::Household => doc.household.retain(|x| x.id != t.entity_id),
            TombstoneEntityKind::PaymentMethod => doc.payment_methods.retain(|x| x.id != t.entity_id),
            TombstoneEntityKind::Tag => doc.tags.retain(|x| x.id != t.entity_id),
        }
    }
}

/// Merge app rows plus deletion tombstones (sync payload). Persists merged tombstones via caller.
pub(crate) fn sync_merge_with_tombstones(
    local: AppDataDoc,
    remote: AppDataDoc,
    local_ts: Vec<DeletionTombstone>,
    remote_ts: Vec<DeletionTombstone>,
) -> Result<(AppDataDoc, Vec<DeletionTombstone>), String> {
    let mut merged = merge_app_data_rows(local, remote);
    let merged_ts = merge_tombstone_vecs(local_ts, remote_ts);
    apply_tombstone_filter(&mut merged, &merged_ts);
    Ok((merged, merged_ts))
}

fn merge_timestamped_by_id<T, FId, FTs>(local: Vec<T>, remote: Vec<T>, get_id: FId, get_ts: FTs) -> Vec<T>
where
    T: Clone,
    FId: Fn(&T) -> String,
    FTs: Fn(&T) -> i64,
{
    let mut out: HashMap<String, T> = HashMap::new();
    for row in local {
        out.insert(get_id(&row), row);
    }
    for row in remote {
        let id = get_id(&row);
        let should_replace = match out.get(&id) {
            Some(existing) => get_ts(&row) >= get_ts(existing),
            None => true,
        };
        if should_replace {
            out.insert(id, row);
        }
    }
    out.into_values().collect()
}

fn merge_by_id_prefer_remote<T, FId>(local: Vec<T>, remote: Vec<T>, get_id: FId) -> Vec<T>
where
    T: Clone,
    FId: Fn(&T) -> String,
{
    let mut out: HashMap<String, T> = HashMap::new();
    for row in local {
        out.insert(get_id(&row), row);
    }
    for row in remote {
        out.insert(get_id(&row), row);
    }
    out.into_values().collect()
}

#[tauri::command]
pub fn sync_build_push_meta(local_updated_at: i64, device_id: String) -> Result<SyncMeta, String> {
    let now = chrono::Utc::now().timestamp_millis();
    let updated_at = if local_updated_at > 0 { local_updated_at } else { now };
    Ok(SyncMeta {
        last_synced_at: now,
        updated_at,
        device_id: device_id.clone(),
        revision: Some(format!("rev_{}_{}", now, device_id)),
        schema_version: Some(2),
    })
}

#[tauri::command]
pub fn sync_payload_fits_limit(payload: SyncPayload, max_bytes: usize) -> Result<bool, String> {
    let bytes = encode_sync_payload(&payload)?;
    Ok(bytes.len() <= max_bytes)
}

/// True if remote file revision does not match the last revision this device recorded (`SyncConfig.remote_revision`).
/// When local revision is empty but remote has a revision, returns true (must pull or force-push).
pub(crate) fn sync_push_revision_mismatch(expected_revision: &str, remote: Option<&SyncPayload>) -> bool {
    let Some(remote) = remote else {
        return false;
    };
    let actual = remote.meta.revision.as_deref().unwrap_or("").trim();
    if actual.is_empty() {
        return false;
    }
    let expected = expected_revision.trim();
    if expected.is_empty() {
        return true;
    }
    expected != actual
}


#[tauri::command]
pub async fn sync_get_ui_schema(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SyncUiSchemaDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::Init).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_get_settings(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncConfig, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::GetConfig).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_save_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    provider: Option<SyncProviderType>,
    credentials: Option<SyncCredentialsDto>,
) -> Result<SyncConfig, String> {
    if let Some(p) = provider {
        sync_dispatch_internal(
            app.clone(),
            state.clone(),
            SyncDispatchCommand::SetCredentials {
                provider: p,
                credentials: credentials.unwrap_or_default(),
            },
        )
        .await?;
    }
    sync_get_settings(app, state).await
}

#[tauri::command]
pub async fn sync_get_status(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncStatusDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::GetStatus).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_enable_provider(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    provider: SyncProviderType,
) -> Result<SyncEnableDto, String> {
    let response = sync_dispatch_internal(
        app,
        state,
        SyncDispatchCommand::Enable { provider },
    )
    .await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_disable_provider(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::Disable).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_oauth_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    provider: SyncProviderType,
) -> Result<SyncEnableDto, String> {
    sync_enable_provider(app, state, provider).await
}

#[tauri::command]
pub async fn sync_oauth_finish(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    code: String,
    provider: Option<SyncProviderType>,
) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(
        app,
        state,
        SyncDispatchCommand::OauthFinish { code, provider },
    )
    .await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_check_remote(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncHasUpdateDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::CheckRemote).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_pull_remote(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::PullRemote).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_push_local(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(
        app,
        state,
        SyncDispatchCommand::PushLocal {
            force_overwrite_remote: false,
        },
    )
    .await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_force_push_local(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(
        app,
        state,
        SyncDispatchCommand::PushLocal {
            force_overwrite_remote: true,
        },
    )
    .await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_now(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::SyncNow).await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_flush_before_exit(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    timeout_ms: Option<u64>,
) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(
        app,
        state,
        SyncDispatchCommand::FlushBeforeExit {
            timeout_ms: timeout_ms.unwrap_or(2500),
        },
    )
    .await?;
    extract_data(response)
}

#[tauri::command]
pub async fn sync_dismiss_pending_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SyncOkDto, String> {
    let response = sync_dispatch_internal(app, state, SyncDispatchCommand::DismissPending).await?;
    extract_data(response)
}

#[cfg(test)]
mod tests {
    use super::{sync_merge_app_data, sync_merge_with_tombstones, sync_push_revision_mismatch};

    #[test]
    fn sync_merge_local_tombstone_drops_remote_row_when_delete_is_newer() {
        use crate::models::{DeletionTombstone, TombstoneEntityKind};

        let base = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        let mut local = base.clone();
        local.subscriptions.clear();

        let mut remote = base.clone();
        for s in &mut remote.subscriptions {
            s.id = "tomb-a".to_string();
            s.updated_at = 50;
            break;
        }

        let ts = vec![DeletionTombstone {
            entity_kind: TombstoneEntityKind::Subscription,
            entity_id: "tomb-a".to_string(),
            deleted_at: 200,
            device_id: "dev".to_string(),
        }];

        let (merged, _) = sync_merge_with_tombstones(local, remote, ts, vec![]).expect("merge");
        assert!(!merged.subscriptions.iter().any(|s| s.id == "tomb-a"));
    }

    #[test]
    fn sync_wire_roundtrip_v2_binary_only() {
        use super::{decode_sync_payload, encode_sync_payload};
        use crate::commands::sync::types::{SyncMeta, SyncPayload};

        let data = crate::commands::seed::seed_get_default_data().expect("seed");
        let payload = SyncPayload {
            data,
            app_config: crate::models::AppConfigDoc::default(),
            meta: SyncMeta {
                last_synced_at: 1,
                updated_at: 2,
                device_id: "d".into(),
                revision: Some("r1".into()),
                schema_version: Some(2),
            },
            tombstones: vec![],
        };
        let encoded = encode_sync_payload(&payload).expect("encode");
        let decoded = decode_sync_payload(&encoded).expect("decode wire");
        assert_eq!(decoded.meta.device_id, payload.meta.device_id);
    }

    #[test]
    fn sync_push_revision_mismatch_detects_remote_drift() {
        use crate::commands::sync::types::{SyncMeta, SyncPayload};

        let data = crate::commands::seed::seed_get_default_data().expect("seed");
        let meta = SyncMeta {
            last_synced_at: 1,
            updated_at: 2,
            device_id: "d".into(),
            revision: Some("remote_rev_b".into()),
            schema_version: Some(2),
        };
        let payload = SyncPayload {
            data,
            app_config: crate::models::AppConfigDoc::default(),
            meta,
            tombstones: vec![],
        };
        assert!(sync_push_revision_mismatch("remote_rev_a", Some(&payload)));
        assert!(!sync_push_revision_mismatch("remote_rev_b", Some(&payload)));
        assert!(sync_push_revision_mismatch("", Some(&payload)));
        assert!(!sync_push_revision_mismatch("x", None));
    }

    #[test]
    fn sync_merge_prefers_latest_updated_at_for_user_entities() {
        let mut local = crate::test_support::doc_with_restart_sensitive_fields().expect("local doc");
        let mut remote = local.clone();

        local.subscriptions[0].id = "sub-merge-1".to_string();
        local.subscriptions[0].updated_at = 100;
        local.subscriptions[0].name = "local-sub".to_string();

        remote.subscriptions[0].id = "sub-merge-1".to_string();
        remote.subscriptions[0].updated_at = 200;
        remote.subscriptions[0].name = "remote-sub".to_string();

        local.expenses[0].id = "exp-merge-1".to_string();
        local.expenses[0].updated_at = 300;
        local.expenses[0].name = "local-exp".to_string();

        remote.expenses[0].id = "exp-merge-1".to_string();
        remote.expenses[0].updated_at = 250;
        remote.expenses[0].name = "remote-exp".to_string();

        let merged = sync_merge_app_data(local.clone(), remote.clone()).expect("merge");

        let merged_sub = merged
            .subscriptions
            .iter()
            .find(|x| x.id == "sub-merge-1")
            .expect("merged subscription");
        assert_eq!(merged_sub.name, "remote-sub", "newer remote subscription should win");

        let merged_exp = merged
            .expenses
            .iter()
            .find(|x| x.id == "exp-merge-1")
            .expect("merged expense");
        assert_eq!(merged_exp.name, "local-exp", "newer local expense should win");
    }
}
