//! `ai_extract_receipt` — vision-LLM receipt parser.
//!
//! `aisdk` 0.5 doesn't expose multimodal input, so this module bypasses it
//! and POSTs an OpenAI-compatible chat-completions request directly via
//! `reqwest`. The provider configuration (base_url + API key) is still
//! resolved through [`super::super::providers::load_active_provider`] so the
//! same OpenRouter / Ollama / Custom endpoint applies.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

use crate::commands::ai::dto::{ExpenseDraftDto, ExpenseLineItem};
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{apply_common, resolve_amount, resolve_date};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::providers::{self, ActiveProvider};
use crate::commands::ai::shared::{require_feature_enabled, AiFeature};
use crate::errors::AppError;
use crate::AppState;

const SUPPORTED_IMAGE_MIME: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/webp",
    "image/gif",
];

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiReceiptRaw {
    #[serde(flatten)]
    common: AiCommonFields,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    line_items: Vec<AiReceiptLineItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiReceiptLineItem {
    #[serde(default)]
    name: String,
    #[serde(default)]
    amount: Option<f64>,
}

#[tauri::command]
pub async fn ai_extract_receipt(
    state: State<'_, AppState>,
    bytes: Vec<u8>,
    mime: String,
    locale: Option<String>,
) -> Result<ExpenseDraftDto, AppError> {
    require_feature_enabled(AiFeature::ReceiptImport)?;

    if bytes.is_empty() {
        return Err(AppError::from("ai_receipt_empty"));
    }

    let ctx = ExtractContext::from_state(&state, locale)?;
    let mime_lc = mime.to_lowercase();

    let raw = if mime_lc == "application/pdf" || mime_lc == "pdf" {
        extract_pdf(&bytes, &ctx).await?
    } else if is_supported_image(&mime_lc) {
        extract_image(&bytes, &mime_lc, &ctx).await?
    } else if mime_lc == "image/heic" || mime_lc == "image/heif" {
        // Phase 5: try to transcode HEIC via the `image` crate (works when
        // the build is linked against libheif). Falls back to a friendly
        // error if the platform image stack doesn't support it.
        let (jpeg_bytes, jpeg_mime) = transcode_heic_to_jpeg(&bytes)?;
        extract_image(&jpeg_bytes, jpeg_mime, &ctx).await?
    } else {
        return Err(AppError::from(format!("ai_receipt_unsupported_mime:{mime}")));
    };

    Ok(map_draft(raw, &ctx))
}

fn is_supported_image(mime: &str) -> bool {
    SUPPORTED_IMAGE_MIME
        .iter()
        .any(|m| m.eq_ignore_ascii_case(mime))
}

/// Image path: send the bytes as a base64 data URL to the OpenAI-compatible
/// chat-completions endpoint.
async fn extract_image(
    bytes: &[u8],
    mime: &str,
    ctx: &ExtractContext,
) -> Result<AiReceiptRaw, AppError> {
    let active = providers::load_active_provider()?;
    let system = crate::commands::ai::prompts::receipt::build(ctx);
    let response_text = call_vision_chat(&active, &system, bytes, mime).await?;
    parse_llm_json::<AiReceiptRaw>(&response_text)
}

/// PDF path: try local text extraction first; if the PDF carries no embedded
/// text (likely a scan), return an actionable error so the UI can suggest
/// photographing the receipt instead.
async fn extract_pdf(bytes: &[u8], ctx: &ExtractContext) -> Result<AiReceiptRaw, AppError> {
    let text = pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| AppError::from(format!("ai_receipt_pdf_extract:{e}")))?;
    if text.trim().len() < 30 {
        return Err(AppError::from("ai_receipt_pdf_scan_unsupported"));
    }
    let system = crate::commands::ai::prompts::receipt::build(ctx);
    let prompt = format!("Receipt text extracted from PDF:\n```\n{text}\n```");
    let response_text = providers::run_text(&system, &prompt).await?;
    parse_llm_json::<AiReceiptRaw>(&response_text)
}

/// Attempt to transcode HEIC bytes to JPEG using the `image` crate.
///
/// HEIC support in the `image` crate requires linking against libheif at
/// build time, which we don't enable (it's a heavy native dependency that
/// doesn't ship on every CI host). We *still* try `load_from_memory`
/// because some platform image stacks (notably macOS / iOS via Tauri's
/// own decoders) sometimes succeed for individual frames. If decoding
/// fails we surface an actionable error so the UI can ask the user to
/// re-export the image as JPEG.
fn transcode_heic_to_jpeg(bytes: &[u8]) -> Result<(Vec<u8>, &'static str), AppError> {
    let dyn_image = image::load_from_memory(bytes)
        .map_err(|_| AppError::from("ai_receipt_heic_unsupported"))?;
    let rgb = dyn_image.to_rgb8();
    let mut out = std::io::Cursor::new(Vec::new());
    rgb.write_to(&mut out, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::from(format!("ai_receipt_heic_encode_failed:{e}")))?;
    Ok((out.into_inner(), "image/jpeg"))
}

/// Direct chat-completions call for vision models. Returns the raw assistant
/// content string (we then parse JSON from it).
async fn call_vision_chat(
    active: &ActiveProvider,
    system: &str,
    bytes: &[u8],
    mime: &str,
) -> Result<String, AppError> {
    let url = format!("{}/chat/completions", active.base_url.trim_end_matches('/'));
    let data_url = format!("data:{};base64,{}", mime, B64.encode(bytes));

    let body = json!({
        "model": active.model,
        "temperature": 0,
        "max_tokens": 800,
        "messages": [
            { "role": "system", "content": system },
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "Extract the receipt details as JSON. Reply with ONLY the JSON object."
                    },
                    {
                        "type": "image_url",
                        "image_url": { "url": data_url }
                    }
                ]
            }
        ]
    });

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
        .map_err(|e| AppError::from(format!("ai_receipt_request:{e}")))?;
    let status = resp.status();
    let raw_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::from(format!(
            "ai_receipt_http_{}:{}",
            status.as_u16(),
            truncate(&raw_text, 240)
        )));
    }

    let value: Value = serde_json::from_str(&raw_text)
        .map_err(|e| AppError::from(format!("ai_receipt_response_parse:{e}")))?;
    let content = value
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    if content.is_empty() {
        return Err(AppError::from("ai_receipt_empty_response"));
    }
    Ok(content.to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut out = s[..max].to_string();
        out.push_str("…");
        out
    }
}

fn map_draft(raw: AiReceiptRaw, ctx: &ExtractContext) -> ExpenseDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (amount, amount_warnings) = resolve_amount(raw.amount);
    resolved.warnings.extend(amount_warnings);
    let (date, date_warnings) = resolve_date(raw.date.as_deref());
    resolved.warnings.extend(date_warnings);

    let line_items: Vec<ExpenseLineItem> = raw
        .line_items
        .into_iter()
        .filter_map(|li| {
            let item_name = li.name.trim().to_string();
            let item_amount = li.amount.unwrap_or(0.0).abs();
            if item_name.is_empty() && item_amount <= 0.0 {
                return None;
            }
            Some(ExpenseLineItem {
                name: item_name,
                amount: item_amount,
            })
        })
        .collect();

    ExpenseDraftDto {
        name: resolved.name,
        amount,
        currency_id: resolved.currency_id,
        currency_code: resolved.currency_code,
        date,
        category_id: resolved.category_id,
        category_hint: resolved.category_hint,
        payment_method_id: resolved.payment_method_id,
        tags: Vec::new(),
        notes: resolved.notes,
        url: String::new(),
        line_items,
        warnings: resolved.warnings,
        confidence: resolved.confidence,
        usage: None,
    }
}
