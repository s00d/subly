//! AI provider registry — mirrors `commands/rates/providers/mod.rs`.
//!
//! We expose **two** providers in the UI:
//! * `openrouter`     — proxies to OpenRouter (single key, 200+ models).
//! * `openai_compat`  — any user-supplied OpenAI Chat-Completions endpoint
//!                      (Ollama, LM Studio, vLLM, corporate gateways).
//!
//! Both are routed through the same `aisdk::providers::OpenAICompatible<DynamicModel>`
//! under the hood. The only difference is the default `base_url` and whether
//! the API key is required — see [`AiProviderDescriptor`].
//!
//! Vision-LLM (Phase 5) bypasses aisdk because aisdk 0.5 only accepts text
//! `UserMessage { content: String }`. The receipt module will POST directly
//! to `<base_url>/chat/completions` with multimodal `content` arrays.

pub mod deepseek;
pub mod google_gemini;
pub mod groq;
pub mod mistral;
pub mod openai;
pub mod openai_compat;
pub mod openrouter;
pub mod presets;

use std::time::Instant;

use aisdk::core::LanguageModelRequest;
use aisdk::core::DynamicModel;
use aisdk::providers::OpenAICompatible;

use crate::commands::ai::dto::{AiTestResultDto, AiUsage};
use crate::commands::ai::prompts;
use crate::errors::AppError;
pub use presets::{ModelPreset, ModelPresetDto};

/// Result of a single LLM text generation call. The `usage` field is `None`
/// when the provider didn't surface counters (local Ollama, older LM Studio).
#[derive(Debug, Clone)]
pub struct RunTextResult {
    pub text: String,
    pub usage: Option<AiUsage>,
}

/// Compile-time provider record. Cloned into [`AiProviderMetaDto`] for IPC.
pub struct AiProviderDescriptor {
    pub provider_type: &'static str,
    pub name: &'static str,
    pub default_model: &'static str,
    /// `None` → endpoint must be supplied by the user.
    pub default_base_url: Option<&'static str>,
    pub requires_endpoint: bool,
    pub requires_key: bool,
    pub docs_url: &'static str,
    pub description: &'static str,
    /// Curated dropdown of known-good models, surfaced in the settings UI.
    pub recommended_models: &'static [ModelPreset],
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderMetaDto {
    pub r#type: String,
    pub name: String,
    pub default_model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_base_url: Option<String>,
    pub requires_endpoint: bool,
    pub requires_key: bool,
    pub docs_url: String,
    pub description: String,
    #[serde(default)]
    pub recommended_models: Vec<ModelPresetDto>,
}

pub fn provider_descriptors() -> Vec<AiProviderDescriptor> {
    vec![
        openai::descriptor(),
        openrouter::descriptor(),
        google_gemini::descriptor(),
        groq::descriptor(),
        deepseek::descriptor(),
        mistral::descriptor(),
        openai_compat::descriptor(),
    ]
}

// ---------- Storage key constants -------------------------------------------------

pub mod storage_keys {
    //! Centralised redb / keyring key names so every call site agrees on layout.

    pub const CONFIG_PROVIDER: &str = "config:aiProvider";
    pub const CONFIG_MODEL: &str = "config:aiModel";
    pub const CONFIG_ENDPOINT: &str = "config:aiCustomEndpoint";
    #[allow(dead_code)] // Consumed by Phases 2–5 (per-feature toggles).
    pub const CONFIG_FEATURES: &str = "config:aiFeatures";
    #[allow(dead_code)] // Consumed by Phases 2–5 (master enable flag).
    pub const CONFIG_ENABLED: &str = "config:aiEnabled";

    /// Frontend uses `secure_storage.` prefix (same as [`crate::keyring_store`] accounts); raw key
    /// stored on the frontend is `aiApiKey.<provider>`.
    pub const SECURE_API_KEY_PREFIX: &str = "aiApiKey.";

    /// Direct keyring account name (what `keyring_store::get` expects).
    pub fn full_secure_account(provider_type: &str) -> String {
        format!("secure_storage.{SECURE_API_KEY_PREFIX}{provider_type}")
    }
}

// ---------- Loading active provider + credentials --------------------------------

#[derive(Debug, Clone)]
pub struct ActiveProvider {
    pub provider_type: String,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

pub fn load_active_provider() -> Result<ActiveProvider, AppError> {
    let provider_type = read_string_config(storage_keys::CONFIG_PROVIDER)?
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "openrouter".to_string());

    let descriptor = provider_descriptors()
        .into_iter()
        .find(|d| d.provider_type == provider_type)
        .ok_or_else(|| AppError::from(format!("ai_provider_unknown:{provider_type}")))?;

    let model = read_string_config(storage_keys::CONFIG_MODEL)?
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| descriptor.default_model.to_string());

    let base_url = if descriptor.requires_endpoint {
        let endpoint = read_string_config(storage_keys::CONFIG_ENDPOINT)?
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::from("ai_endpoint_required"))?;
        endpoint.trim_end_matches('/').to_string()
    } else {
        descriptor
            .default_base_url
            .ok_or_else(|| AppError::from("ai_provider_default_url_missing"))?
            .trim_end_matches('/')
            .to_string()
    };

    let api_key = crate::keyring_store::get(&storage_keys::full_secure_account(
        descriptor.provider_type,
    ))?
    .unwrap_or_default();

    if descriptor.requires_key && api_key.trim().is_empty() {
        return Err(AppError::from("ai_api_key_required"));
    }

    Ok(ActiveProvider {
        provider_type: descriptor.provider_type.to_string(),
        model,
        base_url,
        api_key,
    })
}

fn read_string_config(key: &str) -> Result<Option<String>, AppError> {
    let raw = crate::redb_get_internal(key.to_string())?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    // Mirror `config_get`: stored JSON may be a JSON string (preferred) or a
    // raw legacy string. Handle both transparently.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
        match value {
            serde_json::Value::String(s) => Ok(Some(s)),
            serde_json::Value::Null => Ok(None),
            other => Ok(Some(other.to_string())),
        }
    } else {
        Ok(Some(raw))
    }
}

// ---------- aisdk wrapper --------------------------------------------------------

/// Run a single-turn text generation call against the currently-selected
/// provider. Returns text + usage; usage is `None` when the provider didn't
/// report it.
///
/// `system` and `prompt` are both passed verbatim — caller is responsible
/// for any locale/JSON-schema prompt-engineering.
pub async fn run_text_with_usage(system: &str, prompt: &str) -> Result<RunTextResult, AppError> {
    let active = load_active_provider()?;
    let model_name = active.model.clone();
    let provider = build_provider(&active)?;
    let resp = LanguageModelRequest::builder()
        .model(provider)
        .system(system.to_string())
        .prompt(prompt.to_string())
        .temperature(0u32)
        .max_retries(2u32)
        .build()
        .generate_text()
        .await
        .map_err(|e| {
            AppError::from(format!(
                "ai_request_failed:{}:{}",
                model_name,
                friendly_provider_error(&e.to_string())
            ))
        })?;

    let usage = resp.usage();
    let usage = if usage.input_tokens.is_some() || usage.output_tokens.is_some() {
        Some(AiUsage::from_aisdk(&usage))
    } else {
        None
    };
    Ok(RunTextResult {
        text: resp.text().unwrap_or_default(),
        usage,
    })
}

/// Backwards-compatible wrapper around [`run_text_with_usage`] that drops
/// the usage info. Use this when the caller doesn't track token spend.
pub async fn run_text(system: &str, prompt: &str) -> Result<String, AppError> {
    run_text_with_usage(system, prompt).await.map(|r| r.text)
}

fn build_provider(active: &ActiveProvider) -> Result<OpenAICompatible<DynamicModel>, AppError> {
    // aisdk's OpenAICompatibleBuilder mandates a non-empty API key (errors out
    // with `MissingField("api_key")`). Local Ollama-style endpoints typically
    // don't need one, so fall back to a stub literal — the server ignores it.
    let api_key = if active.api_key.trim().is_empty() {
        "no-key".to_string()
    } else {
        active.api_key.clone()
    };

    OpenAICompatible::<DynamicModel>::builder()
        .provider_name(active.provider_type.clone())
        .base_url(active.base_url.clone())
        .api_key(api_key)
        .model_name(active.model.clone())
        .build()
        .map_err(|e| AppError::from(format!("ai_provider_init_failed:{e}")))
}

/// Strip noisy quotes / SDK debug prefixes so the UI can surface a short toast.
fn friendly_provider_error(raw: &str) -> String {
    let mut msg = raw.replace('\n', " ");
    if msg.len() > 240 {
        msg.truncate(240);
        msg.push_str("…");
    }
    msg
}

// ---------- Tauri commands -------------------------------------------------------

#[tauri::command]
pub fn ai_get_providers() -> Result<Vec<AiProviderMetaDto>, AppError> {
    Ok(provider_descriptors()
        .into_iter()
        .map(|d| AiProviderMetaDto {
            r#type: d.provider_type.to_string(),
            name: d.name.to_string(),
            default_model: d.default_model.to_string(),
            default_base_url: d.default_base_url.map(|s| s.to_string()),
            requires_endpoint: d.requires_endpoint,
            requires_key: d.requires_key,
            docs_url: d.docs_url.to_string(),
            description: d.description.to_string(),
            recommended_models: d
                .recommended_models
                .iter()
                .map(ModelPresetDto::from)
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub async fn ai_test_connection() -> Result<AiTestResultDto, AppError> {
    let started = Instant::now();
    let model = load_active_provider().map(|p| p.model).unwrap_or_default();

    match run_text(prompts::TEST_SYSTEM, prompts::TEST_PROMPT).await {
        Ok(text) => {
            let echo: String = text.trim().chars().take(64).collect();
            Ok(AiTestResultDto {
                ok: true,
                latency_ms: started.elapsed().as_millis() as u64,
                echo,
                model,
                error: None,
            })
        }
        Err(err) => Ok(AiTestResultDto {
            ok: false,
            latency_ms: started.elapsed().as_millis() as u64,
            echo: String::new(),
            model,
            error: Some(err.to_string()),
        }),
    }
}
