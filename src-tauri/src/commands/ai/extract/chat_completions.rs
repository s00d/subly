//! Direct OpenAI-compatible `/chat/completions` calls (text + vision).
//!
//! aisdk 0.5 does not expose `response_format: json_object` or multimodal
//! user messages, so structured extraction and vision bypass it here.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde_json::{json, Value};

use crate::commands::ai::dto::AiUsage;
use crate::commands::ai::extract::vision_io::truncate;
use crate::commands::ai::providers::ActiveProvider;
use crate::commands::ai::providers::RunTextResult;
use crate::errors::AppError;

/// `response_format: json_object` is widely supported on cloud OpenAI-compatible APIs.
pub fn provider_supports_json_object(provider_type: &str) -> bool {
    matches!(
        provider_type,
        "openai" | "openrouter" | "gemini" | "groq" | "mistral" | "deepseek" | "openai_compat"
    )
}

/// True when we should retry the same request without `json_object` (local / legacy servers).
pub fn is_json_object_mode_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("response_format")
        || lower.contains("json_object")
        || lower.contains("json schema")
        || (lower.contains("unsupported") && lower.contains("400"))
        || lower.contains("invalid_request_error")
            && (lower.contains("response format") || lower.contains("json"))
}

fn parse_usage(value: &Value) -> Option<AiUsage> {
    let u = value.get("usage")?;
    Some(AiUsage {
        input_tokens: u
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .map(|n| n.min(u32::MAX as u64) as u32),
        output_tokens: u
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .map(|n| n.min(u32::MAX as u64) as u32),
        reasoning_tokens: u
            .get("completion_tokens_details")
            .and_then(|d| d.get("reasoning_tokens"))
            .and_then(|v| v.as_u64())
            .map(|n| n.min(u32::MAX as u64) as u32),
        cached_tokens: u
            .get("prompt_tokens_details")
            .and_then(|d| d.get("cached_tokens"))
            .and_then(|v| v.as_u64())
            .map(|n| n.min(u32::MAX as u64) as u32),
    })
}

fn extract_assistant_text(value: &Value) -> String {
    value
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or_default()
        .to_string()
}

async fn post_chat_completions(
    active: &ActiveProvider,
    body: Value,
    err_prefix: &'static str,
) -> Result<(String, Option<AiUsage>), AppError> {
    let url = format!(
        "{}/chat/completions",
        active.base_url.trim_end_matches('/')
    );

    let mut req = reqwest::Client::new().post(&url);
    if !active.api_key.trim().is_empty() {
        req = req.bearer_auth(active.api_key.trim());
    }
    if active.provider_type == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://github.com/subly")
            .header("X-Title", "Subly");
    }

    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::from(format!("{err_prefix}_request:{e}")))?;
    let status = resp.status();
    let raw_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::from(format!(
            "{err_prefix}_http_{}:{}",
            status.as_u16(),
            truncate(&raw_text, 240)
        )));
    }

    let value: Value = serde_json::from_str(&raw_text)
        .map_err(|e| AppError::from(format!("{err_prefix}_response_parse:{e}")))?;
    let content = extract_assistant_text(&value);
    if content.is_empty() {
        return Err(AppError::from(format!("{err_prefix}_empty_response")));
    }
    Ok((content, parse_usage(&value)))
}

/// Text-only chat completion. When `json_object` is true, asks the API for a JSON object reply.
pub async fn call_text_chat(
    active: &ActiveProvider,
    system: &str,
    user_prompt: &str,
    json_object: bool,
) -> Result<RunTextResult, AppError> {
    let mut body = json!({
        "model": active.model,
        "temperature": 0,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user_prompt }
        ]
    });
    if json_object {
        body["response_format"] = json!({ "type": "json_object" });
    }
    let (text, usage) = post_chat_completions(active, body, "ai_text_chat").await?;
    Ok(RunTextResult { text, usage })
}

/// Multimodal chat completion (image + instruction).
#[allow(clippy::too_many_arguments)]
pub async fn call_vision_chat(
    active: &ActiveProvider,
    system: &str,
    user_instruction: &str,
    bytes: &[u8],
    mime: &str,
    max_tokens: u32,
    err_prefix: &'static str,
    json_object: bool,
) -> Result<String, AppError> {
    let data_url = format!("data:{};base64,{}", mime, B64.encode(bytes));
    let mut body = json!({
        "model": active.model,
        "temperature": 0,
        "max_tokens": max_tokens,
        "messages": [
            { "role": "system", "content": system },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": user_instruction },
                    { "type": "image_url", "image_url": { "url": data_url } }
                ]
            }
        ]
    });
    if json_object {
        body["response_format"] = json!({ "type": "json_object" });
    }
    let (text, _) = post_chat_completions(active, body, err_prefix).await?;
    Ok(text)
}
