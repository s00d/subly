//! Helpers shared between AI command modules.

use serde::Deserialize;

use crate::commands::ai::providers::storage_keys;
use crate::errors::AppError;

/// Reads `mainCurrencyId` from the persisted UI settings blob, returning
/// `None` if the setting is missing or empty.
pub fn read_main_currency_id() -> Option<String> {
    let raw = crate::redb_get_internal("config:settings".to_string())
        .ok()
        .flatten()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let id = value.get("mainCurrencyId")?.as_str()?.to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Read a boolean config value. Mirrors `read_string_config` semantics — the
/// stored blob can be either a JSON boolean or a JSON-encoded string from a
/// previous schema version.
fn read_bool_config(key: &str) -> Result<bool, AppError> {
    let Some(raw) = crate::redb_get_internal(key.to_string())? else {
        return Ok(false);
    };
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
        match value {
            serde_json::Value::Bool(b) => return Ok(b),
            serde_json::Value::String(s) => {
                return Ok(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"));
            }
            _ => {}
        }
    }
    Ok(matches!(raw.to_lowercase().as_str(), "true" | "1" | "yes"))
}

/// Per-feature toggles (mirrors `AiFeatureToggles` on the frontend). All
/// fields default to `true` so a missing key behaves like "feature is
/// available" — the master `aiEnabled` flag is the only kill-switch.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiFeatures {
    #[serde(default = "default_true")]
    pub subscription_input: bool,
    #[serde(default = "default_true")]
    pub expense_input: bool,
    #[serde(default = "default_true")]
    pub statement_import: bool,
    #[serde(default = "default_true")]
    pub receipt_import: bool,
}

impl Default for AiFeatures {
    fn default() -> Self {
        Self {
            subscription_input: true,
            expense_input: true,
            statement_import: true,
            receipt_import: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Strongly-typed identifier of a feature gate.
#[derive(Debug, Clone, Copy)]
pub enum AiFeature {
    SubscriptionInput,
    ExpenseInput,
    StatementImport,
    ReceiptImport,
}

impl AiFeature {
    fn is_enabled(self, features: &AiFeatures) -> bool {
        match self {
            AiFeature::SubscriptionInput => features.subscription_input,
            AiFeature::ExpenseInput => features.expense_input,
            AiFeature::StatementImport => features.statement_import,
            AiFeature::ReceiptImport => features.receipt_import,
        }
    }
}

/// Read the `aiFeatures` blob from config. Falls back to defaults if missing
/// or unparseable — that way a user with an older settings file keeps
/// everything enabled rather than getting silent feature failures.
pub fn read_features() -> Result<AiFeatures, AppError> {
    let Some(raw) = crate::redb_get_internal(storage_keys::CONFIG_FEATURES.to_string())? else {
        return Ok(AiFeatures::default());
    };
    let outer: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
    // Stored blob is either a JSON-stringified object or a direct object —
    // mirror the same dual-encoding tolerance as `read_string_config`.
    let inner = match outer {
        serde_json::Value::String(s) => serde_json::from_str(&s).ok(),
        other if other.is_object() => Some(other),
        _ => None,
    };
    Ok(inner
        .and_then(|v| serde_json::from_value::<AiFeatures>(v).ok())
        .unwrap_or_default())
}

/// Return `Ok(())` only if (a) the AI master switch is on **and** (b) the
/// specific feature is enabled. Otherwise surface a discoverable error code
/// the frontend can map to a localised toast.
///
/// This is the **only** trusted authorisation gate — UI toggles can be
/// bypassed via devtools/IPC, so each `ai_*` command calls this first.
pub fn require_feature_enabled(feature: AiFeature) -> Result<(), AppError> {
    if !read_bool_config(storage_keys::CONFIG_ENABLED)? {
        return Err(AppError::from("ai_disabled"));
    }
    let features = read_features()?;
    if !feature.is_enabled(&features) {
        return Err(AppError::from("ai_feature_disabled"));
    }
    Ok(())
}
