//! Shared DTOs between AI commands and the frontend.
//!
//! Concrete `*Draft` types are camelCase JSON and roughly mirror the
//! corresponding Subscription / Expense forms on the frontend so they can be
//! consumed as `Partial<Subscription>` / `Partial<Expense>` prefills.

use serde::{Deserialize, Serialize};

use crate::models::{CategoryDoc, CurrencyDoc, PaymentMethodDoc, TagDoc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTestResultDto {
    pub ok: bool,
    pub latency_ms: u64,
    #[serde(default)]
    pub echo: String,
    #[serde(default)]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Token usage echoed back from a single LLM call. Surfaces to the frontend
/// via `*Draft.usage` so we can build a "monthly limit" UI without another
/// round trip. All counters are optional because most local OpenAI-compatible
/// servers (Ollama, LM Studio < 0.3) don't populate the `usage` field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
}

impl AiUsage {
    /// Convert from `aisdk::core::language_model::Usage` (which uses `usize`).
    pub fn from_aisdk(u: &aisdk::core::language_model::Usage) -> Self {
        let cap = |o: Option<usize>| o.map(|n| n.min(u32::MAX as usize) as u32);
        Self {
            input_tokens: cap(u.input_tokens),
            output_tokens: cap(u.output_tokens),
            reasoning_tokens: cap(u.reasoning_tokens),
            cached_tokens: cap(u.cached_tokens),
        }
    }
}

/// In-memory snapshot of catalogs taken before issuing an LLM call. Kept in
/// Rust so we can resolve LLM-supplied free-form hints to internal IDs
/// without round-tripping through the frontend.
#[derive(Debug, Clone)]
pub struct CatalogSnapshot {
    pub categories: Vec<CategoryDoc>,
    pub currencies: Vec<CurrencyDoc>,
    pub payment_methods: Vec<PaymentMethodDoc>,
    pub tags: Vec<TagDoc>,
}

/// LLM-extracted subscription draft, ready to seed `SubscriptionForm.prefill`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDraftDto {
    pub name: String,
    pub price: f64,
    /// Empty if the LLM-supplied code did not match any existing currency.
    #[serde(default)]
    pub currency_id: String,
    /// Original code/symbol echoed by the LLM (helps the user identify mistakes).
    #[serde(default)]
    pub currency_code: String,
    /// `1=day, 2=week, 3=month, 4=year` (matches `SubscriptionDoc::cycle`).
    #[serde(default = "default_cycle")]
    pub cycle: u8,
    #[serde(default = "default_frequency")]
    pub frequency: u32,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub category_hint: String,
    #[serde(default)]
    pub payment_method_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_payment: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Non-fatal hints to surface in the UI (e.g. "currency not found, defaulted to USD").
    #[serde(default)]
    pub warnings: Vec<String>,
    /// 0-1 confidence echoed by the model (0 if not provided).
    #[serde(default)]
    pub confidence: f32,
    /// Token usage for this call, when the provider reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<AiUsage>,
}

fn default_cycle() -> u8 {
    3
}
fn default_frequency() -> u32 {
    1
}

/// LLM-extracted single expense — seeds `ExpenseForm.prefill`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseDraftDto {
    pub name: String,
    pub amount: f64,
    #[serde(default)]
    pub currency_id: String,
    #[serde(default)]
    pub currency_code: String,
    /// `YYYY-MM-DD` if the LLM supplied a parseable date; empty otherwise.
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub category_hint: String,
    #[serde(default)]
    pub payment_method_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub url: String,
    /// Optional itemised receipt lines (used by Phase 5).
    #[serde(default)]
    pub line_items: Vec<ExpenseLineItem>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub confidence: f32,
    /// Token usage for this call, when the provider reported it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<AiUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseLineItem {
    pub name: String,
    pub amount: f64,
}

/// What the AI thought it was looking at. Drives the metadata payload and
/// the UI badge in `AiImportDialog`. The row list is treated uniformly —
/// even a `Receipt` carries exactly one row, so frontend code paths never
/// need to branch on `kind` for the save flow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiImportKind {
    /// A single sales receipt: one expense, optional `lineItems` breakdown.
    Receipt,
    /// A list of transactions (bank statement, screenshot of an in-app
    /// transaction history, CSV export, etc.). One row per expense.
    Statement,
}

/// Optional extras that only make sense for one of the [`AiImportKind`]
/// variants. Frontend treats every field as optional and renders the bits
/// it actually has.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiImportMetadata {
    /// Receipt: merchant / store name as printed on the receipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merchant_name: Option<String>,
    /// Receipt: grand total (echoed from the row for convenience).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
    /// Receipt: itemised lines from the bill. Always present (possibly
    /// empty) so the UI can render `lineItems.length > 0` conditionals
    /// without a `?? []` dance.
    #[serde(default)]
    pub line_items: Vec<ExpenseLineItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementDraftRow {
    /// "heuristic" | "llm" — where this row came from.
    pub source: &'static str,
    pub draft: ExpenseDraftDto,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementImportStats {
    pub matched_by_heuristic: usize,
    pub matched_by_llm: usize,
    pub failed: usize,
    pub total: usize,
}

/// Sibling of [`AiImportKind`] used by the subscription branch. The model
/// classifies the input as either one subscription confirmation (`Single`)
/// or a list of active subscriptions (`List`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiSubscriptionImportKind {
    Single,
    List,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSubscriptionDraftRow {
    /// "heuristic" never applies to subscriptions, but kept as a tagged
    /// string so the preview component can stay identical to the expense
    /// one.
    pub source: &'static str,
    pub draft: SubscriptionDraftDto,
}

/// Aggregate result of `ai_smart_input`. The discriminator is `surface` —
/// frontend uses TS's discriminated-union pattern to pick the right
/// preview list and bulk-save handler. Row schemas differ between the two
/// variants, so the shared bits (format, stats, usage) are repeated
/// rather than promoted to a parent struct.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "surface", rename_all = "lowercase")]
pub enum AiSmartResultDto {
    Expense {
        /// `Receipt` for single-item inputs (text, single-receipt photo),
        /// `Statement` for multi-row inputs (CSV, bank screenshot).
        kind: AiImportKind,
        /// `"text" | "image" | "csv" | "xlsx" | "json" | "pdf"`.
        format: &'static str,
        rows: Vec<StatementDraftRow>,
        metadata: AiImportMetadata,
        stats: StatementImportStats,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<AiUsage>,
    },
    Subscription {
        kind: AiSubscriptionImportKind,
        format: &'static str,
        rows: Vec<AiSubscriptionDraftRow>,
        stats: StatementImportStats,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<AiUsage>,
    },
}

/// Streamed progress events sent over `tauri::ipc::Channel`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StatementImportProgress {
    #[serde(rename_all = "camelCase")]
    Detected { format: &'static str },
    #[serde(rename_all = "camelCase")]
    Heuristic { resolved: usize, unresolved: usize },
    #[serde(rename_all = "camelCase")]
    LlmStart { chunks: usize },
    #[serde(rename_all = "camelCase")]
    LlmChunk { index: usize, total: usize },
    #[serde(rename_all = "camelCase")]
    Done { total: usize },
}
