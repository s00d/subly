//! Subscription surface for `ai_smart_input`.
//!
//! Three routes:
//! * [`run_text`] — free-form description ("Telegram Premium 300₽ in
//!   month") → smart prompt → envelope.
//! * [`run_image`] — Google Play / App Store / Stripe screenshot or
//!   receipt → vision-LLM → envelope.
//! * [`run_tabular`] — CSV / XLSX / JSON / PDF; we just extract the
//!   text and let the LLM do the rest (subscriptions in tabular form
//!   are rare and the heuristics module was built for transaction
//!   lists).

use serde::Deserialize;
use tauri::ipc::Channel;

use crate::commands::ai::dto::{
    AiSmartResultDto, AiSubscriptionDraftRow, AiSubscriptionImportKind, AiUsage,
    StatementImportProgress, StatementImportStats, SubscriptionDraftDto,
};
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::mapping::{
    apply_common, resolve_optional_date, resolve_price,
};
use crate::commands::ai::extract::raw::AiCommonFields;
use crate::commands::ai::extract::vision_io::call_vision_chat;
use crate::commands::ai::heuristics;
use crate::commands::ai::json_parse::parse_llm_json;
use crate::commands::ai::prompts::smart::{self, InputKind, Surface};
use crate::commands::ai::providers;
use crate::errors::AppError;

/// Cap on text we ship into the prompt for tabular inputs. The model's
/// context window is finite; ~50 KB ≈ 12 K tokens for typical CSV data,
/// which fits comfortably alongside the system prompt for every provider
/// we ship.
const MAX_TABULAR_TEXT: usize = 50_000;

// ---------------------------------------------------------------------------
// LLM payload structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiSubscriptionRow {
    #[serde(flatten)]
    common: AiCommonFields,
    #[serde(default)]
    price: Option<f64>,
    #[serde(default)]
    cycle: Option<String>,
    #[serde(default)]
    frequency: Option<u32>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    next_payment: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiSubscriptionEnvelope {
    /// `"single"` | `"list"` — defaults to `"list"` so we err on the safe
    /// (multi-row) side when the model forgets the discriminator.
    #[serde(default = "default_kind")]
    kind: String,
    #[serde(default)]
    subscriptions: Vec<AiSubscriptionRow>,
}

fn default_kind() -> String {
    "list".to_string()
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

    let system = smart::build(ctx, Surface::Subscription, InputKind::Text);
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
    let system = smart::build(ctx, Surface::Subscription, InputKind::Image);
    let response_text = call_vision_chat(
        &active,
        &system,
        "Classify the picture and extract every subscription. Reply with ONLY the JSON envelope.",
        bytes,
        mime,
        4096,
        "ai_import_subscription",
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
    let format = tabular_format_label(mime, bytes);
    let _ = on_progress.send(StatementImportProgress::Detected { format });

    let text = extract_tabular_text(bytes, mime)?;
    if text.trim().is_empty() {
        return Err(AppError::from("ai_import_subscription_no_text"));
    }
    let trimmed = truncate_for_prompt(&text, MAX_TABULAR_TEXT);

    let _ = on_progress.send(StatementImportProgress::LlmStart { chunks: 1 });
    let _ = on_progress.send(StatementImportProgress::LlmChunk { index: 1, total: 1 });

    let system = smart::build(ctx, Surface::Subscription, InputKind::Text);
    let result = providers::run_text_with_usage(&system, &trimmed).await?;
    let envelope = parse_envelope(&result.text)?;
    Ok(finalise(envelope, format, ctx, on_progress, result.usage))
}

// ---------------------------------------------------------------------------
// Envelope → result shaping
// ---------------------------------------------------------------------------

fn finalise(
    envelope: AiSubscriptionEnvelope,
    format: &'static str,
    ctx: &ExtractContext,
    on_progress: &Channel<StatementImportProgress>,
    usage: Option<AiUsage>,
) -> AiSmartResultDto {
    let kind = match envelope.kind.as_str() {
        "single" => AiSubscriptionImportKind::Single,
        _ => AiSubscriptionImportKind::List,
    };

    let mut stats = StatementImportStats::default();
    let mut rows = Vec::<AiSubscriptionDraftRow>::new();
    for row in envelope.subscriptions {
        let draft = row_to_draft(row, ctx);
        if draft.name.is_empty() && draft.price <= 0.0 {
            stats.failed += 1;
            continue;
        }
        rows.push(AiSubscriptionDraftRow {
            source: "llm",
            draft,
        });
        stats.matched_by_llm += 1;
    }
    stats.total = rows.len();
    let _ = on_progress.send(StatementImportProgress::Done { total: stats.total });

    AiSmartResultDto::Subscription {
        kind,
        format,
        rows,
        stats,
        usage,
    }
}

/// Tolerant envelope parser. Same fallback ladder as the expense side:
/// canonical envelope → bare `subscriptions` field → bare array → bare
/// single-object → final error.
fn parse_envelope(raw: &str) -> Result<AiSubscriptionEnvelope, AppError> {
    if let Ok(envelope) = parse_llm_json::<AiSubscriptionEnvelope>(raw) {
        if !envelope.subscriptions.is_empty() {
            return Ok(envelope);
        }
    }
    if let Ok(rows) = parse_llm_json::<Vec<AiSubscriptionRow>>(raw) {
        if !rows.is_empty() {
            return Ok(AiSubscriptionEnvelope {
                kind: default_kind(),
                subscriptions: rows,
            });
        }
    }
    if let Ok(single) = parse_llm_json::<AiSubscriptionRow>(raw) {
        if !single.common.name.trim().is_empty() || single.price.is_some() {
            return Ok(AiSubscriptionEnvelope {
                kind: "single".to_string(),
                subscriptions: vec![single],
            });
        }
    }
    parse_llm_json::<AiSubscriptionEnvelope>(raw)
}

// ---------------------------------------------------------------------------
// Mapping
// ---------------------------------------------------------------------------

fn row_to_draft(raw: AiSubscriptionRow, ctx: &ExtractContext) -> SubscriptionDraftDto {
    let mut resolved = apply_common(&raw.common, ctx);
    let (price, price_warnings) = resolve_price(raw.price);
    resolved.warnings.extend(price_warnings);

    let cycle = map_cycle(raw.cycle.as_deref());
    let frequency = raw.frequency.unwrap_or(1).max(1);
    let start_date = resolve_optional_date(raw.start_date.as_deref());
    let next_payment = resolve_optional_date(raw.next_payment.as_deref());

    let confidence = if raw.common.confidence.is_some() {
        resolved.confidence
    } else {
        0.5
    };

    SubscriptionDraftDto {
        name: resolved.name,
        price,
        currency_id: resolved.currency_id,
        currency_code: resolved.currency_code,
        cycle,
        frequency,
        category_id: resolved.category_id,
        category_hint: resolved.category_hint,
        payment_method_id: resolved.payment_method_id,
        start_date,
        next_payment,
        notes: resolved.notes,
        url: raw.url.unwrap_or_default(),
        tags: raw.tags.unwrap_or_default(),
        warnings: resolved.warnings,
        confidence,
        usage: None,
    }
}

/// Map the model's `cycle` string to the numeric code used by
/// [`SubscriptionDoc`](crate::models::SubscriptionDoc). Defaults to
/// month (3) when the input is missing or unrecognised — the same
/// fallback `default_cycle()` produces in the DTO.
fn map_cycle(raw: Option<&str>) -> u8 {
    let v = raw.map(|s| s.trim().to_ascii_lowercase()).unwrap_or_default();
    match v.as_str() {
        "day" | "daily" | "d" => 1,
        "week" | "weekly" | "w" => 2,
        "year" | "yearly" | "annual" | "annually" | "y" => 4,
        _ => 3, // month is the default per `default_cycle()`.
    }
}

// ---------------------------------------------------------------------------
// Tabular helpers
// ---------------------------------------------------------------------------

/// Best-effort label of the tabular format we got. Re-uses
/// [`heuristics::detect_format`] so the value we emit on the
/// `Detected` progress event matches what the expense pipeline uses.
fn tabular_format_label(mime: &str, bytes: &[u8]) -> &'static str {
    heuristics::detect_format(mime, bytes)
}

/// Extract plain text from a tabular payload. We piggy-back on the
/// heuristics parser because it already knows how to read CSV / XLSX /
/// JSON / PDF and gives us back rows we can flatten into a text block
/// the LLM can read.
fn extract_tabular_text(bytes: &[u8], mime: &str) -> Result<String, AppError> {
    let heur = heuristics::run(bytes, mime)?;
    let mut out = String::new();
    for row in &heur.rows {
        // Heuristic rows already split fields — re-join with " | " to give
        // the model something it can lex as columns.
        let line = row.raw_columns.join(" | ");
        if !line.trim().is_empty() {
            out.push_str(&line);
            out.push('\n');
        }
    }
    // Anything heuristics couldn't structure (PDF prose, free-form text)
    // is appended verbatim so we don't lose subscription rows hiding in
    // weirdly-shaped data.
    for line in &heur.unresolved {
        if !line.trim().is_empty() {
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}

/// Cap a prompt body at `max` bytes, appending a marker so the model
/// knows the input was truncated rather than incomplete on purpose.
fn truncate_for_prompt(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Snap to the last newline before the limit so we don't slice a row
    // in half and confuse the LLM.
    let cut = s[..max].rfind('\n').unwrap_or(max);
    let mut out = s[..cut].to_string();
    out.push_str("\n…[truncated]");
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_cycle_known_values() {
        assert_eq!(map_cycle(Some("day")), 1);
        assert_eq!(map_cycle(Some("WEEK")), 2);
        assert_eq!(map_cycle(Some("month")), 3);
        assert_eq!(map_cycle(Some("Yearly")), 4);
        assert_eq!(map_cycle(Some("annual")), 4);
    }

    #[test]
    fn map_cycle_defaults_to_month() {
        assert_eq!(map_cycle(None), 3);
        assert_eq!(map_cycle(Some("")), 3);
        assert_eq!(map_cycle(Some("weird")), 3);
    }

    #[test]
    fn envelope_canonical_list() {
        let raw = r#"{
            "kind":"list",
            "subscriptions":[
                {"name":"Netflix","price":15.49,"currencyCode":"USD","cycle":"month"},
                {"name":"Spotify","price":9.99,"currencyCode":"USD","cycle":"month"}
            ]
        }"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "list");
        assert_eq!(env.subscriptions.len(), 2);
    }

    #[test]
    fn envelope_canonical_single() {
        let raw = r#"{
            "kind":"single",
            "subscriptions":[
                {"name":"Telegram Premium","price":300,"currencyCode":"RUB","cycle":"month"}
            ]
        }"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "single");
        assert_eq!(env.subscriptions.len(), 1);
    }

    #[test]
    fn envelope_falls_back_to_list_when_kind_missing() {
        let raw = r#"{"subscriptions":[{"name":"X","price":1,"currencyCode":"USD"}]}"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "list");
        assert_eq!(env.subscriptions.len(), 1);
    }

    #[test]
    fn envelope_accepts_bare_array() {
        let raw = r#"[{"name":"Apple One","price":19.95,"currencyCode":"USD","cycle":"month"}]"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "list");
        assert_eq!(env.subscriptions.len(), 1);
    }

    #[test]
    fn envelope_accepts_single_object_as_single() {
        let raw = r#"{"name":"YouTube Premium","price":11.99,"currencyCode":"USD"}"#;
        let env = parse_envelope(raw).unwrap();
        assert_eq!(env.kind, "single");
        assert_eq!(env.subscriptions.len(), 1);
        assert_eq!(env.subscriptions[0].common.name, "YouTube Premium");
    }

    #[test]
    fn envelope_rejects_garbage() {
        assert!(parse_envelope("not json").is_err());
    }

    #[test]
    fn truncate_for_prompt_keeps_short_input_untouched() {
        assert_eq!(truncate_for_prompt("hello", 100), "hello");
    }

    #[test]
    fn truncate_for_prompt_snaps_to_newline() {
        let body = "row1\nrow2\nrow3-very-long-row-that-overflows";
        let truncated = truncate_for_prompt(body, 15);
        assert!(truncated.starts_with("row1\nrow2"));
        assert!(truncated.ends_with("[truncated]"));
        // Did not slice mid-row:
        assert!(truncated.contains("row1\nrow2\n"));
    }
}
