//! `ai_extract_subscription_from_text` — natural-language → SubscriptionDraft.
//!
//! Thin shell around the shared [`ExtractContext`] + [`apply_common`] pipeline.
//! Only feature-specific concerns (price, cycle/frequency, start/next-payment
//! dates, tags) live here.

use serde::Deserialize;
use tauri::State;

use crate::commands::ai::dto::SubscriptionDraftDto;
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{apply_common, resolve_optional_date, resolve_price};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::providers;
use crate::commands::ai::shared::{require_feature_enabled, AiFeature};
use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiSubscriptionRaw {
    #[serde(flatten)]
    common: AiCommonFields,
    #[serde(default)]
    price: Option<f64>,
    /// `day` | `week` | `month` | `year`.
    #[serde(default)]
    cycle: Option<String>,
    #[serde(default)]
    frequency: Option<u32>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    next_payment: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[tauri::command]
pub async fn ai_extract_subscription_from_text(
    state: State<'_, AppState>,
    text: String,
    locale: Option<String>,
) -> Result<SubscriptionDraftDto, AppError> {
    require_feature_enabled(AiFeature::SubscriptionInput)?;

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::from("ai_input_empty"));
    }

    let ctx = ExtractContext::from_state(&state, locale)?;
    let system = crate::commands::ai::prompts::subscription::build(&ctx);
    let result = providers::run_text_with_usage(&system, trimmed).await?;
    let raw: AiSubscriptionRaw = parse_llm_json(&result.text)?;

    let mut draft = map_draft(raw, &ctx);
    draft.usage = result.usage;
    Ok(draft)
}

fn map_draft(raw: AiSubscriptionRaw, ctx: &ExtractContext) -> SubscriptionDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (price, price_warnings) = resolve_price(raw.price);
    resolved.warnings.extend(price_warnings);

    SubscriptionDraftDto {
        name: resolved.name,
        price,
        currency_id: resolved.currency_id,
        currency_code: resolved.currency_code,
        cycle: map_cycle(raw.cycle.as_deref()),
        frequency: raw.frequency.unwrap_or(1).max(1),
        category_id: resolved.category_id,
        category_hint: resolved.category_hint,
        payment_method_id: resolved.payment_method_id,
        start_date: resolve_optional_date(raw.start_date.as_deref()),
        next_payment: resolve_optional_date(raw.next_payment.as_deref()),
        notes: resolved.notes,
        url: raw.url.unwrap_or_default(),
        tags: raw.tags.unwrap_or_default(),
        warnings: resolved.warnings,
        confidence: resolved.confidence,
        usage: None,
    }
}

/// Maps `day|week|month|year` strings (LLM wire format) to the internal
/// `1=day, 2=week, 3=month, 4=year` enum used by `SubscriptionDoc::cycle`.
fn map_cycle(raw: Option<&str>) -> u8 {
    let Some(value) = raw else { return 3 };
    match value.trim().to_lowercase().as_str() {
        "day" | "daily" | "days" | "d" => 1,
        "week" | "weekly" | "weeks" | "w" => 2,
        "year" | "yearly" | "annual" | "years" | "y" => 4,
        _ => 3,
    }
}
