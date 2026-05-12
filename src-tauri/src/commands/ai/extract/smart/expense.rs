//! Expense surface for `ai_smart_input`.
//!
//! Three routes:
//! * [`run_text`] — free-form description → smart prompt → envelope.
//! * [`run_image`] — picture (receipt or transaction-list screenshot)
//!   → vision-LLM → envelope.
//! * [`run_tabular`] — CSV / XLSX / JSON / text PDF → heuristics rows
//!   resolved offline, unresolved lines bundled into LLM chunks via the
//!   legacy statement prompt.
//!
//! Whichever route fires, the output is a uniform
//! [`AiSmartResultDto::Expense`] envelope so the frontend stays oblivious.

use serde::Deserialize;
use tauri::ipc::Channel;

use crate::commands::ai::dto::{
    AiImportKind, AiImportMetadata, AiSmartResultDto, AiUsage, CatalogSnapshot, ExpenseDraftDto,
    ExpenseLineItem, StatementDraftRow, StatementImportProgress, StatementImportStats,
};
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{apply_common, resolve_amount, resolve_date};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::extract::vision_io::call_vision_chat;
use crate::commands::ai::heuristics::{self, HeuristicResult, HeuristicRow};
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::prompts::smart::{self, InputKind, Surface};
use crate::commands::ai::providers;
use crate::errors::AppError;

/// Max raw lines bundled into one heuristics-fallback LLM batch. Same value
/// used by the previous statement importer — kept small so the prompt stays
/// inside the context window of low-tier local models.
const LLM_CHUNK_SIZE: usize = 30;

// ---------------------------------------------------------------------------
// LLM payload structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiExpenseRow {
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiExpenseEnvelope {
    /// `"receipt"` | `"statement"` — defaults to `"statement"` so a model
    /// that ignores the field still gets the safer multi-row treatment.
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    transactions: Vec<AiExpenseRow>,
    #[serde(default)]
    metadata: AiExpenseMetadataRaw,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiExpenseMetadataRaw {
    #[serde(default)]
    merchant_name: Option<String>,
    #[serde(default)]
    total_amount: Option<f64>,
    #[serde(default)]
    line_items: Vec<AiLineItemRaw>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiLineItemRaw {
    #[serde(default)]
    name: String,
    #[serde(default)]
    amount: Option<f64>,
}

/// Mini envelope used by the heuristics-fallback chunk LLM call. Same
/// shape as the canonical [`AiExpenseEnvelope`] minus the discriminator
/// and metadata — the chunk only returns rows.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiStatementBatch {
    #[serde(default)]
    transactions: Vec<AiExpenseRow>,
}

fn default_kind() -> String {
    "statement".to_string()
}

// ---------------------------------------------------------------------------
// Public entry points (called by `mod.rs` dispatcher)
// ---------------------------------------------------------------------------

pub(super) async fn run_text(
    text: &str,
    ctx: &ExtractContext,
    on_progress: &Channel<StatementImportProgress>,
) -> Result<AiSmartResultDto, AppError> {
    let _ = on_progress.send(StatementImportProgress::Detected { format: "text" });
    let _ = on_progress.send(StatementImportProgress::LlmStart { chunks: 1 });
    let _ = on_progress.send(StatementImportProgress::LlmChunk { index: 1, total: 1 });

    let system = smart::build(ctx, Surface::Expense, InputKind::Text);
    let result = providers::run_text_with_usage(&system, text).await?;
    let envelope = parse_envelope(&result.text)?;
    Ok(finalise(envelope, "text", ctx, on_progress, result.usage))
}

pub(super) async fn run_image(
    bytes: &[u8],
    mime: &str,
    ctx: &ExtractContext,
    on_progress: &Channel<StatementImportProgress>,
) -> Result<AiSmartResultDto, AppError> {
    let _ = on_progress.send(StatementImportProgress::Detected { format: "image" });
    let _ = on_progress.send(StatementImportProgress::LlmStart { chunks: 1 });
    let _ = on_progress.send(StatementImportProgress::LlmChunk { index: 1, total: 1 });

    let active = providers::load_active_provider()?;
    let system = smart::build(ctx, Surface::Expense, InputKind::Image);
    let response_text = call_vision_chat(
        &active,
        &system,
        "Classify the picture and extract its transactions. Reply with ONLY the JSON envelope.",
        bytes,
        mime,
        4096,
        "ai_import",
    )
    .await?;

    let envelope = parse_envelope(&response_text)?;
    Ok(finalise(envelope, "image", ctx, on_progress, None))
}

pub(super) async fn run_tabular(
    bytes: &[u8],
    mime: &str,
    ctx: &ExtractContext,
    on_progress: &Channel<StatementImportProgress>,
) -> Result<AiSmartResultDto, AppError> {
    let heur = heuristics::run(bytes, mime)?;
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

    let mut total_usage: Option<AiUsage> = None;
    if !heur.unresolved.is_empty() {
        let chunks = chunk_lines(&heur.unresolved, LLM_CHUNK_SIZE);
        let _ = on_progress.send(StatementImportProgress::LlmStart {
            chunks: chunks.len(),
        });
        let total = chunks.len();
        for (i, chunk) in chunks.into_iter().enumerate() {
            let _ = on_progress.send(StatementImportProgress::LlmChunk {
                index: i + 1,
                total,
            });
            match call_llm_chunk(ctx, &heur, &chunk).await {
                Ok((parsed, usage)) => {
                    if let Some(u) = usage {
                        total_usage = Some(merge_usage(total_usage.take(), u));
                    }
                    for row in parsed.transactions {
                        let draft = row_to_draft(row, ctx);
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
                    eprintln!("[subly::ai::smart::expense] LLM chunk failed: {e}");
                    stats.failed += chunk.len();
                }
            }
        }
    }

    stats.total = rows.len();
    let _ = on_progress.send(StatementImportProgress::Done { total: stats.total });

    Ok(AiSmartResultDto::Expense {
        kind: AiImportKind::Statement,
        format: heur.format,
        rows,
        metadata: AiImportMetadata::default(),
        stats,
        usage: total_usage,
    })
}

// ---------------------------------------------------------------------------
// Envelope → result shaping
// ---------------------------------------------------------------------------

fn finalise(
    envelope: AiExpenseEnvelope,
    format: &'static str,
    ctx: &ExtractContext,
    on_progress: &Channel<StatementImportProgress>,
    usage: Option<AiUsage>,
) -> AiSmartResultDto {
    let kind = match envelope.kind.as_str() {
        "receipt" => AiImportKind::Receipt,
        _ => AiImportKind::Statement,
    };

    let mut stats = StatementImportStats::default();
    let mut rows = Vec::<StatementDraftRow>::new();
    for row in envelope.transactions {
        let draft = row_to_draft(row, ctx);
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
    stats.total = rows.len();

    let metadata = build_metadata(&envelope.metadata);
    if kind == AiImportKind::Receipt {
        if let Some(first) = rows.first_mut() {
            first.draft.line_items = metadata.line_items.clone();
        }
    }

    let _ = on_progress.send(StatementImportProgress::Done { total: stats.total });

    AiSmartResultDto::Expense {
        kind,
        format,
        rows,
        metadata,
        stats,
        usage,
    }
}

/// Tolerant envelope parser. Accepts the canonical `{ kind, transactions,
/// metadata }` shape and three looser fallbacks the model might emit when
/// it ignores parts of the schema.
fn parse_envelope(raw: &str) -> Result<AiExpenseEnvelope, AppError> {
    if let Ok(envelope) = parse_llm_json::<AiExpenseEnvelope>(raw) {
        if !envelope.transactions.is_empty() || envelope.metadata.merchant_name.is_some() {
            return Ok(envelope);
        }
    }
    if let Ok(batch) = parse_llm_json::<AiStatementBatch>(raw) {
        if !batch.transactions.is_empty() {
            return Ok(AiExpenseEnvelope {
                kind: default_kind(),
                transactions: batch.transactions,
                metadata: AiExpenseMetadataRaw::default(),
            });
        }
    }
    if let Ok(rows) = parse_llm_json::<Vec<AiExpenseRow>>(raw) {
        if !rows.is_empty() {
            return Ok(AiExpenseEnvelope {
                kind: default_kind(),
                transactions: rows,
                metadata: AiExpenseMetadataRaw::default(),
            });
        }
    }
    if let Ok(single) = parse_llm_json::<AiExpenseRow>(raw) {
        if !single.common.name.trim().is_empty() || single.amount.is_some() {
            return Ok(AiExpenseEnvelope {
                kind: "receipt".to_string(),
                transactions: vec![single],
                metadata: AiExpenseMetadataRaw::default(),
            });
        }
    }
    parse_llm_json::<AiExpenseEnvelope>(raw)
}

fn build_metadata(raw: &AiExpenseMetadataRaw) -> AiImportMetadata {
    let line_items: Vec<ExpenseLineItem> = raw
        .line_items
        .iter()
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
    AiImportMetadata {
        merchant_name: raw.merchant_name.clone().filter(|s| !s.trim().is_empty()),
        total_amount: raw.total_amount,
        line_items,
    }
}

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn row_to_draft(raw: AiExpenseRow, ctx: &ExtractContext) -> ExpenseDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (amount, amount_warnings) = resolve_amount(raw.amount);
    resolved.warnings.extend(amount_warnings);
    let (date, date_warnings) = resolve_date(raw.date.as_deref());
    resolved.warnings.extend(date_warnings);

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
        tags: raw.tags.unwrap_or_default(),
        notes: resolved.notes,
        url: raw.url.unwrap_or_default(),
        line_items: Vec::new(),
        warnings: resolved.warnings,
        confidence,
        usage: None,
    }
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

// ---------------------------------------------------------------------------
// Heuristics-fallback helpers
// ---------------------------------------------------------------------------

/// A heuristic-parsed row is "clean" when we already have a name, a
/// positive amount, and a date — enough to drop it straight into the form
/// without an LLM round-trip.
fn row_is_clean(row: &HeuristicRow) -> bool {
    !row.name.trim().is_empty() && row.amount > 0.0 && !row.date.is_empty()
}

fn chunk_lines(lines: &[String], size: usize) -> Vec<Vec<String>> {
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
    let body = chunk.join("\n");
    let prompt = format!("Raw bank statement lines:\n```\n{body}\n```");
    let result = providers::run_text_with_usage(&system, &prompt).await?;
    let parsed = parse_llm_json::<AiStatementBatch>(&result.text)?;
    Ok((parsed, result.usage))
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(chunk_lines(&[], 5).is_empty());
    }

    #[test]
    fn chunk_lines_handles_zero_size() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let chunks = chunk_lines(&lines, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 2);
    }

    #[test]
    fn envelope_canonical() {
        let raw = r#"{
            "kind":"receipt",
            "transactions":[
                {"name":"Magnit","amount":1234.56,"currencyCode":"RUB","date":"2026-05-01"}
            ],
            "metadata":{
                "merchantName":"Magnit",
                "totalAmount":1234.56,
                "lineItems":[
                    {"name":"Milk","amount":99.0},
                    {"name":"Bread","amount":45.0}
                ]
            }
        }"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "receipt");
        assert_eq!(env.transactions.len(), 1);
        assert_eq!(env.metadata.line_items.len(), 2);
        assert_eq!(env.metadata.merchant_name.as_deref(), Some("Magnit"));
    }

    #[test]
    fn envelope_falls_back_to_statement_when_kind_missing() {
        let raw = r#"{
            "transactions":[
                {"name":"Starbucks","amount":5.4,"currencyCode":"USD","date":"2026-04-30"}
            ]
        }"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "statement");
        assert_eq!(env.transactions.len(), 1);
    }

    #[test]
    fn envelope_accepts_bare_array() {
        let raw = r#"[
            {"name":"Uber","amount":12.3,"currencyCode":"USD","date":"2026-04-29"}
        ]"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "statement");
        assert_eq!(env.transactions.len(), 1);
    }

    #[test]
    fn envelope_accepts_single_object() {
        let raw = r#"{"name":"Coffee","amount":3.5,"currencyCode":"USD","date":"2026-05-10"}"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "receipt");
        assert_eq!(env.transactions.len(), 1);
        assert_eq!(env.transactions[0].common.name, "Coffee");
    }

    #[test]
    fn envelope_rejects_garbage() {
        assert!(parse_envelope("not json at all").is_err());
    }

    #[test]
    fn build_metadata_filters_empty_lines() {
        let raw = AiExpenseMetadataRaw {
            merchant_name: Some("Whole Foods".to_string()),
            total_amount: Some(42.5),
            line_items: vec![
                AiLineItemRaw {
                    name: "Bananas".to_string(),
                    amount: Some(3.5),
                },
                AiLineItemRaw {
                    name: "".to_string(),
                    amount: Some(0.0),
                },
                AiLineItemRaw {
                    name: "  ".to_string(),
                    amount: None,
                },
            ],
        };
        let meta = build_metadata(&raw);
        assert_eq!(meta.merchant_name.as_deref(), Some("Whole Foods"));
        assert_eq!(meta.line_items.len(), 1);
        assert_eq!(meta.line_items[0].name, "Bananas");
    }

    #[test]
    fn merge_usage_sums_counters() {
        let a = AiUsage {
            input_tokens: Some(10),
            output_tokens: Some(5),
            reasoning_tokens: None,
            cached_tokens: Some(2),
        };
        let b = AiUsage {
            input_tokens: Some(3),
            output_tokens: None,
            reasoning_tokens: Some(7),
            cached_tokens: None,
        };
        let merged = merge_usage(Some(a), b);
        assert_eq!(merged.input_tokens, Some(13));
        assert_eq!(merged.output_tokens, Some(5));
        assert_eq!(merged.reasoning_tokens, Some(7));
        assert_eq!(merged.cached_tokens, Some(2));
    }
}
