//! Subscription login/password/TOTP in the OS keyring (never in redb / entity rows).
//!
//! On every list reload the UI also needs to know **which** credential fields
//! are filled (to render reveal/copy buttons) without actually pulling the
//! secret out of the keyring — that would trigger an OS password prompt for
//! every row on macOS. So we mirror a tiny non-secret bitmap into redb at
//! `idx:subscription_creds:{id}` ([`SubscriptionCredentialsMetaDto`]) any
//! time we save / delete / reveal credentials, and the list command reads
//! that index instead of touching the keyring.

use base64::Engine;
use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

use crate::models::{SubscriptionCredentialsDto, SubscriptionCredentialsMetaDto};
use crate::state::AppStateInner;
use crate::AppState;
use tauri::State;

const SECURE_PREFIX: &str = "secure_storage.";

fn credentials_storage_account(subscription_id: &str) -> String {
    let id = subscription_id.trim();
    format!("{SECURE_PREFIX}subscription_credentials:{id}")
}

pub(crate) fn creds_meta_index_key(subscription_id: &str) -> String {
    format!("idx:subscription_creds:{}", subscription_id.trim())
}

/// Compute the non-secret bitmap from a freshly-resolved credentials blob.
fn meta_from_dto(dto: &SubscriptionCredentialsDto) -> SubscriptionCredentialsMetaDto {
    SubscriptionCredentialsMetaDto {
        has_login: !dto.login.trim().is_empty(),
        has_password: !dto.password.is_empty(),
        has_totp: !dto.totp_secret.trim().is_empty(),
    }
}

fn meta_is_empty(meta: &SubscriptionCredentialsMetaDto) -> bool {
    !meta.has_login && !meta.has_password && !meta.has_totp
}

/// Write (or clear) the credentials metadata index for `subscription_id`.
/// Empty meta erases the row instead of storing `{false,false,false}` so a
/// scan in the future can treat "missing key" and "no creds" identically.
pub(crate) fn write_meta_index(
    state: &AppStateInner,
    subscription_id: &str,
    meta: &SubscriptionCredentialsMetaDto,
) -> Result<(), crate::errors::AppError> {
    let key = creds_meta_index_key(subscription_id);
    if meta_is_empty(meta) {
        state.redb_delete(&key)?;
        return Ok(());
    }
    let json = serde_json::to_string(meta)
        .map_err(|e| crate::errors::AppError::from(format!("creds_meta_encode:{e}")))?;
    state.redb_set(&key, &json)
}

/// Read the non-secret credentials bitmap from redb. Returns the zero value
/// when the key is missing or malformed — these cases mean "no creds known".
pub(crate) fn read_meta_index(
    state: &AppStateInner,
    subscription_id: &str,
) -> Result<SubscriptionCredentialsMetaDto, crate::errors::AppError> {
    let key = creds_meta_index_key(subscription_id);
    let Some(raw) = state.redb_get(&key)? else {
        return Ok(SubscriptionCredentialsMetaDto::default());
    };
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub(crate) fn credentials_read(
    subscription_id: &str,
) -> Result<Option<SubscriptionCredentialsDto>, crate::errors::AppError> {
    let account = credentials_storage_account(subscription_id);
    let Some(raw) = crate::keyring_store::get(&account)? else {
        return Ok(None);
    };
    serde_json::from_str::<SubscriptionCredentialsDto>(&raw)
        .map(Some)
        .map_err(|e| {
            crate::errors::AppError::from(format!("credentials decode failed: {e}"))
        })
}

pub(crate) fn credentials_delete(
    state: &AppStateInner,
    subscription_id: &str,
) -> Result<(), crate::errors::AppError> {
    crate::keyring_store::delete(&credentials_storage_account(subscription_id))?;
    state.redb_delete(&creds_meta_index_key(subscription_id))?;
    Ok(())
}

/// When `credentials` is `None`, leaves secure storage unchanged (caller omitted field).
/// When `Some`, replaces secure blob or deletes if all fields empty. Either
/// branch refreshes the redb meta index so the next list reload reflects
/// the new state without touching the keyring.
pub(crate) fn credentials_apply_optional(
    state: &AppStateInner,
    subscription_id: &str,
    credentials: Option<SubscriptionCredentialsDto>,
) -> Result<(), crate::errors::AppError> {
    let Some(dto) = credentials else {
        return Ok(());
    };
    let login = dto.login.trim().to_string();
    let totp = dto.totp_secret.trim().to_string();
    let password = dto.password;
    if login.is_empty() && password.is_empty() && totp.is_empty() {
        credentials_delete(state, subscription_id)?;
        return Ok(());
    }
    save_credentials_json(
        state,
        subscription_id,
        &SubscriptionCredentialsDto {
            login,
            password,
            totp_secret: totp,
        },
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionTotpCurrentDto {
    pub code: String,
    pub period_sec: u64,
    pub valid_until_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpauthImportDto {
    pub totp_secret: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub issuer: String,
}

fn save_credentials_json(
    state: &AppStateInner,
    subscription_id: &str,
    dto: &SubscriptionCredentialsDto,
) -> Result<(), crate::errors::AppError> {
    let account = credentials_storage_account(subscription_id);
    let json = serde_json::to_string(dto)?;
    crate::keyring_store::set(&account, &json)?;
    write_meta_index(state, subscription_id, &meta_from_dto(dto))?;
    Ok(())
}

fn totp_from_secret_b32(secret_b32: &str) -> Result<TOTP, crate::errors::AppError> {
    let cleaned: String = secret_b32.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err(crate::errors::AppError::from("totp_secret_missing"));
    }
    let upper = cleaned.to_uppercase();
    let bytes = Secret::Encoded(upper)
        .to_bytes()
        .map_err(|e| crate::errors::AppError::from(format!("invalid_totp_secret:{e}")))?;
    TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes).map_err(|e| {
        crate::errors::AppError::from(format!("totp_init:{e}"))
    })
}

/// Next Unix ms when the current period rolls (exclusive end for displayed code).
fn valid_until_ms_for_step(step_sec: u64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let next = ((now / step_sec) + 1) * step_sec;
    (next.saturating_mul(1000)) as i64
}

#[tauri::command]
pub fn subscription_credentials_get(
    state: State<'_, AppState>,
    subscription_id: String,
) -> Result<Option<SubscriptionCredentialsDto>, crate::errors::AppError> {
    let creds = credentials_read(&subscription_id)?;
    // Self-heal the redb index — if the user reveals a credential on a
    // subscription that pre-dates the meta index (or whose index drifted
    // from the keyring), we update it on the spot.
    if let Some(ref c) = creds {
        let guard = state
            .lock()
            .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        write_meta_index(&guard, &subscription_id, &meta_from_dto(c))?;
    }
    Ok(creds)
}

#[tauri::command]
pub fn subscription_credentials_set(
    state: State<'_, AppState>,
    subscription_id: String,
    dto: SubscriptionCredentialsDto,
) -> Result<(), crate::errors::AppError> {
    let guard = state
        .lock()
        .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    save_credentials_json(&guard, &subscription_id, &dto)
}

#[tauri::command]
pub fn subscription_credentials_delete(
    state: State<'_, AppState>,
    subscription_id: String,
) -> Result<(), crate::errors::AppError> {
    let guard = state
        .lock()
        .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    credentials_delete(&guard, &subscription_id)
}

#[tauri::command]
pub fn subscription_totp_current(
    state: State<'_, AppState>,
    subscription_id: String,
) -> Result<SubscriptionTotpCurrentDto, crate::errors::AppError> {
    let creds = credentials_read(&subscription_id)?.ok_or_else(|| {
        crate::errors::AppError::from("no_credentials")
    })?;
    // Same self-heal — first OTP poll after a fresh install can repair the
    // index from the keyring without re-saving.
    {
        let guard = state
            .lock()
            .map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        write_meta_index(&guard, &subscription_id, &meta_from_dto(&creds))?;
    }
    let totp = totp_from_secret_b32(&creds.totp_secret)?;
    let step = totp.step;
    let code = totp
        .generate_current()
        .map_err(|e| crate::errors::AppError::from(format!("totp_time:{e}")))?;
    Ok(SubscriptionTotpCurrentDto {
        code,
        period_sec: step,
        valid_until_ms: valid_until_ms_for_step(step),
    })
}

/// Parse `otpauth://totp/...` or `otpauth://hotp/...` (TOTP only; hotp returns error).
#[tauri::command]
pub fn subscription_totp_import_otpauth(uri: String) -> Result<OtpauthImportDto, crate::errors::AppError> {
    let u = Url::parse(uri.trim()).map_err(|e| {
        crate::errors::AppError::from(format!("invalid_otpauth_url:{e}"))
    })?;
    if u.scheme() != "otpauth" {
        return Err(crate::errors::AppError::from("otpauth_scheme_required"));
    }
    let typ = u.host_str().unwrap_or("").to_ascii_lowercase();
    if typ != "totp" {
        return Err(crate::errors::AppError::from("totp_only_supported"));
    }

    let get_q = |key: &str| -> Option<String> {
        u.query_pairs()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.into_owned())
    };

    let secret = get_q("secret").ok_or_else(|| {
        crate::errors::AppError::from("otpauth_missing_secret")
    })?;
    if secret.trim().is_empty() {
        return Err(crate::errors::AppError::from("otpauth_empty_secret"));
    }

    let issuer_q = get_q("issuer").unwrap_or_default();

    let path = u.path().trim_start_matches('/');
    let label_raw = percent_decode_str(path)
        .decode_utf8_lossy()
        .into_owned();
    let (label, issuer_from_path) = split_label_and_issuer(&label_raw, &issuer_q);

    Ok(OtpauthImportDto {
        totp_secret: secret.chars().filter(|c| !c.is_whitespace()).collect(),
        label,
        issuer: issuer_from_path,
    })
}

fn split_label_and_issuer(label_raw: &str, issuer_query: &str) -> (String, String) {
    if !issuer_query.is_empty() {
        return (label_raw.to_string(), issuer_query.to_string());
    }
    if let Some((a, b)) = label_raw.split_once(':') {
        return (b.trim().to_string(), a.trim().to_string());
    }
    (label_raw.to_string(), String::new())
}

/// PNG/JPEG/GIF image as standard base64 or data-URL; decode QR and parse otpauth payload.
#[tauri::command]
pub fn subscription_totp_decode_qr_base64(data_base64: String) -> Result<OtpauthImportDto, crate::errors::AppError> {
    let b64 = strip_data_url_prefix(&data_base64);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|e| crate::errors::AppError::from(format!("invalid_base64:{e}")))?;

    let img = image::load_from_memory(&bytes).map_err(|e| {
        crate::errors::AppError::from(format!("invalid_image:{e}"))
    })?;
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    let grid = grids.first().ok_or_else(|| crate::errors::AppError::from("qr_not_found"))?;
    let (_meta, content) = grid.decode().map_err(|e| {
        crate::errors::AppError::from(format!("qr_decode:{e}"))
    })?;
    let content = content.trim();
    if content.starts_with("otpauth://") {
        return subscription_totp_import_otpauth(content.to_string());
    }
    Err(crate::errors::AppError::from("qr_not_otpauth"))
}

fn strip_data_url_prefix(s: &str) -> &str {
    let s = s.trim();
    if let Some(idx) = s.find(',') {
        if s[..idx].contains("base64") {
            return &s[idx + 1..];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_state() -> AppStateInner {
        let (dir, db) = crate::test_support::temp_db().expect("temp db");
        let doc = crate::test_support::doc_with_restart_sensitive_fields().expect("doc");
        // Mirror the production invariant: `lib::init_kv_table` runs before
        // any command, so reads against an empty KV table don't blow up.
        {
            let tx = db.begin_write().expect("write tx");
            let _ = tx.open_table(crate::KV_TABLE).expect("open kv");
            tx.commit().expect("commit");
        }
        std::mem::forget(dir);
        AppStateInner {
            db: std::sync::Arc::new(db),
            app_data: doc,
        }
    }

    #[test]
    fn meta_index_round_trip() {
        let state = temp_state();
        let meta = SubscriptionCredentialsMetaDto {
            has_login: true,
            has_password: false,
            has_totp: true,
        };
        write_meta_index(&state, "sub-1", &meta).expect("write");
        let read = read_meta_index(&state, "sub-1").expect("read");
        assert!(read.has_login);
        assert!(!read.has_password);
        assert!(read.has_totp);
    }

    #[test]
    fn meta_index_empty_meta_clears_key() {
        let state = temp_state();
        write_meta_index(
            &state,
            "sub-1",
            &SubscriptionCredentialsMetaDto {
                has_login: true,
                has_password: true,
                has_totp: true,
            },
        )
        .expect("write");
        // Now overwrite with all-false → key should disappear.
        write_meta_index(&state, "sub-1", &SubscriptionCredentialsMetaDto::default())
            .expect("clear");
        // Direct redb check: the key is gone, not stored as `{false,false,false}`.
        let raw = state
            .redb_get(&creds_meta_index_key("sub-1"))
            .expect("redb_get");
        assert!(raw.is_none(), "expected key to be deleted, got {raw:?}");
        // And `read_meta_index` returns the zero value.
        let read = read_meta_index(&state, "sub-1").expect("read");
        assert!(!read.has_login && !read.has_password && !read.has_totp);
    }

    #[test]
    fn meta_index_missing_key_is_zero() {
        let state = temp_state();
        let read = read_meta_index(&state, "never-saved").expect("read");
        assert!(!read.has_login && !read.has_password && !read.has_totp);
    }

    #[test]
    fn meta_from_dto_treats_whitespace_login_as_missing() {
        let m = meta_from_dto(&SubscriptionCredentialsDto {
            login: "   ".to_string(),
            password: "pw".to_string(),
            totp_secret: "\t".to_string(),
        });
        assert!(!m.has_login);
        assert!(m.has_password);
        assert!(!m.has_totp);
    }
}
