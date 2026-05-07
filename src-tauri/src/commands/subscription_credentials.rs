//! Subscription login/password/TOTP in the OS keyring (never in redb / entity rows).

use base64::Engine;
use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

use crate::models::SubscriptionCredentialsDto;
const SECURE_PREFIX: &str = "secure_storage.";

fn credentials_storage_account(subscription_id: &str) -> String {
    let id = subscription_id.trim();
    format!("{SECURE_PREFIX}subscription_credentials:{id}")
}

pub(crate) fn credentials_read(subscription_id: &str) -> Result<Option<SubscriptionCredentialsDto>, String> {
    let account = credentials_storage_account(subscription_id);
    let Some(raw) = crate::keyring_store::get(&account)? else {
        return Ok(None);
    };
    serde_json::from_str::<SubscriptionCredentialsDto>(&raw)
        .map(Some)
        .map_err(|e| format!("credentials decode failed: {e}"))
}

pub(crate) fn credentials_delete(subscription_id: &str) -> Result<(), String> {
    crate::keyring_store::delete(&credentials_storage_account(subscription_id))
}

/// When `credentials` is `None`, leaves secure storage unchanged (caller omitted field).
/// When `Some`, replaces secure blob or deletes if all fields empty.
pub(crate) fn credentials_apply_optional(
    subscription_id: &str,
    credentials: Option<SubscriptionCredentialsDto>,
) -> Result<(), String> {
    let Some(dto) = credentials else {
        return Ok(());
    };
    let login = dto.login.trim().to_string();
    let totp = dto.totp_secret.trim().to_string();
    let password = dto.password;
    if login.is_empty() && password.is_empty() && totp.is_empty() {
        credentials_delete(subscription_id)?;
        return Ok(());
    }
    save_credentials_json(
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

fn save_credentials_json(subscription_id: &str, dto: &SubscriptionCredentialsDto) -> Result<(), String> {
    let account = credentials_storage_account(subscription_id);
    let json = serde_json::to_string(dto).map_err(|e| e.to_string())?;
    crate::keyring_store::set(&account, &json)
}

fn totp_from_secret_b32(secret_b32: &str) -> Result<TOTP, String> {
    let cleaned: String = secret_b32.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err("totp_secret_missing".to_string());
    }
    let upper = cleaned.to_uppercase();
    let bytes = Secret::Encoded(upper)
        .to_bytes()
        .map_err(|e| format!("invalid_totp_secret:{e}"))?;
    TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes).map_err(|e| format!("totp_init:{e}"))
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
pub fn subscription_credentials_get(subscription_id: String) -> Result<Option<SubscriptionCredentialsDto>, String> {
    credentials_read(&subscription_id)
}

#[tauri::command]
pub fn subscription_credentials_set(subscription_id: String, dto: SubscriptionCredentialsDto) -> Result<(), String> {
    save_credentials_json(&subscription_id, &dto)
}

#[tauri::command]
pub fn subscription_credentials_delete(subscription_id: String) -> Result<(), String> {
    credentials_delete(&subscription_id)
}

#[tauri::command]
pub fn subscription_totp_current(subscription_id: String) -> Result<SubscriptionTotpCurrentDto, String> {
    let creds = credentials_read(&subscription_id)?.ok_or_else(|| "no_credentials".to_string())?;
    let totp = totp_from_secret_b32(&creds.totp_secret)?;
    let step = totp.step;
    let code = totp
        .generate_current()
        .map_err(|e| format!("totp_time:{e}"))?;
    Ok(SubscriptionTotpCurrentDto {
        code,
        period_sec: step,
        valid_until_ms: valid_until_ms_for_step(step),
    })
}

/// Parse `otpauth://totp/...` or `otpauth://hotp/...` (TOTP only; hotp returns error).
#[tauri::command]
pub fn subscription_totp_import_otpauth(uri: String) -> Result<OtpauthImportDto, String> {
    let u = Url::parse(uri.trim()).map_err(|e| format!("invalid_otpauth_url:{e}"))?;
    if u.scheme() != "otpauth" {
        return Err("otpauth_scheme_required".to_string());
    }
    let typ = u.host_str().unwrap_or("").to_ascii_lowercase();
    if typ != "totp" {
        return Err("totp_only_supported".to_string());
    }

    let get_q = |key: &str| -> Option<String> {
        u.query_pairs()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.into_owned())
    };

    let secret = get_q("secret").ok_or_else(|| "otpauth_missing_secret".to_string())?;
    if secret.trim().is_empty() {
        return Err("otpauth_empty_secret".to_string());
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
pub fn subscription_totp_decode_qr_base64(data_base64: String) -> Result<OtpauthImportDto, String> {
    let b64 = strip_data_url_prefix(&data_base64);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|e| format!("invalid_base64:{e}"))?;

    let img = image::load_from_memory(&bytes).map_err(|e| format!("invalid_image:{e}"))?;
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    let grid = grids.first().ok_or_else(|| "qr_not_found".to_string())?;
    let (_meta, content) = grid.decode().map_err(|e| format!("qr_decode:{e}"))?;
    let content = content.trim();
    if content.starts_with("otpauth://") {
        return subscription_totp_import_otpauth(content.to_string());
    }
    Err("qr_not_otpauth".to_string())
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
