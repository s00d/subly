//! `ai_import_statement_file` — turn a bank statement file (CSV / XLSX /
//! JSON / PDF / plain text) into draft expenses.
//!
//! Pipeline (mirrors the rates provider chain):
//!   * Stage A — heuristics ([`crate::commands::ai::heuristics`]). Cheap,
//!     offline. Resolves rows where a header-based mapping was good enough.
//!   * Stage B — LLM fallback for rows the heuristic couldn't parse.
//!     Batched to keep prompts < ~4 K tokens.
//!
//! Stage C (vision-LLM for scanned PDFs) is intentionally absent — we route
//! scanned PDFs to `ai_extract_receipt` via a clear error code.

use serde::Deserialize;
use tauri::ipc::Channel;
use tauri::State;

use crate::commands::ai::dto::{
    AiUsage,
    CatalogSnapshot,
    ExpenseDraftDto,
    StatementDraftRow,
    StatementImportProgress,
    StatementImportResultDto,
    StatementImportStats,
};
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{apply_common, resolve_amount, resolve_date};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::heuristics::{self, HeuristicResult, HeuristicRow};
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::providers;
use crate::commands::ai::shared::{require_feature_enabled, AiFeature};
use crate::errors::AppError;
use crate::AppState;

/// Max number of raw lines bundled into one LLM batch. Keeping it small so
/// the prompt stays well below context limits for low-end local models.
pub(crate) const LLM_CHUNK_SIZE: usize = 30;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiStatementBatch {
    #[serde(default)]
    transactions: Vec<AiStatementRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiStatementRow {
    #[serde(flatten)]
    common: AiCommonFields,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(default)]
    date: Option<String>,
}

#[tauri::command]
pub async fn ai_import_statement_file(
    state: State<'_, AppState>,
    bytes: Vec<u8>,
    mime: String,
    locale: Option<String>,
    on_progress: Channel<StatementImportProgress>,
) -> Result<StatementImportResultDto, AppError> {
    require_feature_enabled(AiFeature::StatementImport)?;

    if bytes.is_empty() {
        return Err(AppError::from("ai_statement_empty"));
    }

    let ctx = ExtractContext::from_state(&state, locale)?;

    // Stage A: heuristics.
    let heur = heuristics::run(&bytes, &mime)?;
    let _ = on_progress.send(StatementImportProgress::Detected { format: heur.format });

    let mut rows = Vec::<StatementDraftRow>::new();
    let mut stats = StatementImportStats::default();

    for row in &heur.rows {
        if row_is_clean(row) {
            rows.push(StatementDraftRow {
                source: "heuristic",
                draft: heuristic_to_draft(row, &ctx.catalogs, &ctx.main_currency_id),
            });
            stats.matched_by_heuristic += 1;
        }
    }
    let _ = on_progress.send(StatementImportProgress::Heuristic {
        resolved: stats.matched_by_heuristic,
        unresolved: heur.unresolved.len(),
    });

    // Stage B: LLM fallback for unresolved lines.
    let mut total_usage: Option<AiUsage> = None;
    if !heur.unresolved.is_empty() {
        let chunks = chunk_lines(&heur.unresolved, LLM_CHUNK_SIZE);
        let _ = on_progress.send(StatementImportProgress::LlmStart { chunks: chunks.len() });
        let total = chunks.len();
        for (i, chunk) in chunks.into_iter().enumerate() {
            let _ = on_progress.send(StatementImportProgress::LlmChunk { index: i + 1, total });
            match call_llm_chunk(&ctx, &heur, &chunk).await {
                Ok((parsed, usage)) => {
                    if let Some(u) = usage {
                        total_usage = Some(merge_usage(total_usage.take(), u));
                    }
                    for row in parsed.transactions {
                        let draft = llm_row_to_draft(row, &ctx);
                        if draft.name.is_empty() && draft.amount <= 0.0 {
                            stats.failed += 1;
                            continue;
                        }
                        rows.push(StatementDraftRow {
                            source: "llm",
                            draft,
                        });
                        stats.matched_by_llm += 1;
                    }
                }
                Err(e) => {
                    eprintln!("[subly::ai::statement] LLM chunk failed: {e}");
                    stats.failed += chunk.len();
                }
            }
        }
    }

    stats.total = rows.len();
    let _ = on_progress.send(StatementImportProgress::Done { total: stats.total });

    Ok(StatementImportResultDto {
        format: heur.format,
        rows,
        stats,
        usage: total_usage,
    })
}

/// Add the counters from `next` into `acc`. Used to accumulate per-chunk
/// LLM usage into a single statement-import total.
fn merge_usage(acc: Option<AiUsage>, next: AiUsage) -> AiUsage {
    let acc = acc.unwrap_or_default();
    let add = |a: Option<u32>, b: Option<u32>| match (a, b) {
        (Some(x), Some(y)) => Some(x.saturating_add(y)),
        (Some(x), None) | (None, Some(x)) => Some(x),
        _ => None,
    };
    AiUsage {
        input_tokens: add(acc.input_tokens, next.input_tokens),
        output_tokens: add(acc.output_tokens, next.output_tokens),
        reasoning_tokens: add(acc.reasoning_tokens, next.reasoning_tokens),
        cached_tokens: add(acc.cached_tokens, next.cached_tokens),
    }
}

/// A heuristic-parsed row is "clean" when we already have a name, a positive
/// amount, and a date — those three are enough to drop it into the form
/// without an LLM round-trip.
pub(crate) fn row_is_clean(row: &HeuristicRow) -> bool {
    !row.name.trim().is_empty() && row.amount > 0.0 && !row.date.is_empty()
}

fn heuristic_to_draft(
    row: &HeuristicRow,
    catalogs: &CatalogSnapshot,
    main_currency_id: &str,
) -> ExpenseDraftDto {
    let mut warnings = row.warnings.clone();

    let currency_id = catalogs.resolve_currency_id(&row.currency_code, main_currency_id);
    if currency_id.is_empty() {
        warnings.push("ai_warning_currency_unresolved".to_string());
    } else if !row.currency_code.is_empty()
        && !catalogs
            .currencies
            .iter()
            .any(|c| c.code.eq_ignore_ascii_case(&row.currency_code))
    {
        warnings.push("ai_warning_currency_substituted".to_string());
    }

    ExpenseDraftDto {
        name: row.name.clone(),
        amount: row.amount,
        currency_id,
        currency_code: row.currency_code.clone(),
        date: row.date.clone(),
        category_id: String::new(),
        category_hint: String::new(),
        payment_method_id: String::new(),
        tags: Vec::new(),
        notes: row.notes.clone(),
        url: String::new(),
        line_items: Vec::new(),
        warnings,
        confidence: 1.0,
        usage: None,
    }
}

fn llm_row_to_draft(raw: AiStatementRow, ctx: &ExtractContext) -> ExpenseDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (amount, amount_warnings) = resolve_amount(raw.amount);
    resolved.warnings.extend(amount_warnings);
    let (date, date_warnings) = resolve_date(raw.date.as_deref());
    resolved.warnings.extend(date_warnings);

    // Statement rows default to a softer 0.5 confidence when the model omits
    // the field — they're never as trustworthy as a heuristic-parsed row.
    let confidence = if raw.common.confidence.is_some() {
        resolved.confidence
    } else {
        0.5
    };

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
        line_items: Vec::new(),
        warnings: resolved.warnings,
        confidence,
        usage: None,
    }
}

pub(crate) fn chunk_lines(lines: &[String], size: usize) -> Vec<Vec<String>> {
    if size == 0 {
        return vec![lines.to_vec()];
    }
    let mut out = Vec::with_capacity(lines.len().div_ceil(size));
    let mut i = 0;
    while i < lines.len() {
        let end = usize::min(i + size, lines.len());
        out.push(lines[i..end].to_vec());
        i = end;
    }
    out
}

async fn call_llm_chunk(
    ctx: &ExtractContext,
    heur: &HeuristicResult,
    chunk: &[String],
) -> Result<(AiStatementBatch, Option<AiUsage>), AppError> {
    let system = crate::commands::ai::prompts::statement::build(ctx, heur);
    let prompt = build_user_prompt(chunk);
    let result = providers::run_text_with_usage(&system, &prompt).await?;
    let parsed = parse_llm_json::<AiStatementBatch>(&result.text)?;
    Ok((parsed, result.usage))
}

fn build_user_prompt(chunk: &[String]) -> String {
    let body = chunk.join("\n");
    format!("Raw bank statement lines:\n```\n{body}\n```")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ai::heuristics::HeuristicRow;

    fn row(name: &str, amount: f64, date: &str) -> HeuristicRow {
        HeuristicRow {
            name: name.to_string(),
            amount,
            currency_code: String::new(),
            date: date.to_string(),
            notes: String::new(),
            raw_columns: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn row_is_clean_requires_all_three() {
        assert!(row_is_clean(&row("Spotify", 9.99, "2026-05-01")));
        assert!(!row_is_clean(&row("", 9.99, "2026-05-01")));
        assert!(!row_is_clean(&row("Spotify", 0.0, "2026-05-01")));
        assert!(!row_is_clean(&row("Spotify", 9.99, "")));
    }

    #[test]
    fn chunk_lines_splits_evenly() {
        let lines: Vec<String> = (0..7).map(|i| format!("line{i}")).collect();
        let chunks = chunk_lines(&lines, 3);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 3);
        assert_eq!(chunks[1].len(), 3);
        assert_eq!(chunks[2].len(), 1);
    }

    #[test]
    fn chunk_lines_handles_empty_input() {
        let chunks = chunk_lines(&[], 5);
        assert!(chunks.is_empty());
    }

    #[test]
    fn chunk_lines_handles_zero_size() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let chunks = chunk_lines(&lines, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 2);
    }
}
