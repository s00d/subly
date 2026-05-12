//! `ai_extract_expense_from_text` — natural-language → ExpenseDraft.
//!
//! Mirrors [`super::subscription`] but targets one-off expenses (no
//! cycle/frequency, has a `date` field).

use serde::Deserialize;
use tauri::State;

use crate::commands::ai::dto::ExpenseDraftDto;
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{apply_common, resolve_amount, resolve_date};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::providers;
use crate::commands::ai::shared::{require_feature_enabled, AiFeature};
use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiExpenseRaw {
    #[serde(flatten)]
    common: AiCommonFields,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    url: Option<String>,
}

#[tauri::command]
pub async fn ai_extract_expense_from_text(
    state: State<'_, AppState>,
    text: String,
    locale: Option<String>,
) -> Result<ExpenseDraftDto, AppError> {
    require_feature_enabled(AiFeature::ExpenseInput)?;

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::from("ai_input_empty"));
    }

    let ctx = ExtractContext::from_state(&state, locale)?;
    let system = crate::commands::ai::prompts::expense::build(&ctx);
    let result = providers::run_text_with_usage(&system, trimmed).await?;
    let raw: AiExpenseRaw = parse_llm_json(&result.text)?;

    let mut draft = map_draft(raw, &ctx);
    draft.usage = result.usage;
    Ok(draft)
}

fn map_draft(raw: AiExpenseRaw, ctx: &ExtractContext) -> ExpenseDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (amount, amount_warnings) = resolve_amount(raw.amount);
    resolved.warnings.extend(amount_warnings);
    let (date, date_warnings) = resolve_date(raw.date.as_deref());
    resolved.warnings.extend(date_warnings);

    ExpenseDraftDto {
        name: resolved.name,
        amount,
        currency_id: resolved.currency_id,
        currency_code: resolved.currency_code,
        date,
        category_id: resolved.category_id,
        category_hint: resolved.category_hint,
        payment_method_id: resolved.payment_method_id,
        tags: raw.tags.unwrap_or_default(),
        notes: resolved.notes,
        url: raw.url.unwrap_or_default(),
        line_items: Vec::new(),
        warnings: resolved.warnings,
        confidence: resolved.confidence,
        usage: None,
    }
}
