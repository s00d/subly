//! AI-driven extraction pipelines.
//!
//! ```text
//!   ┌──────────────────────────────────────────────────┐
//!   │           text / image / file bytes              │
//!   └────────────────────────┬─────────────────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ ExtractContext::from_state │
//!                │  (catalogs + locale + today) │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │  feature-specific prompt │
//!                │   (prompts/<feature>.rs)  │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ providers::run_text /    │
//!                │ direct vision-POST       │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │  json_parse::parse_llm_json │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ apply_common (mapping.rs) │
//!                │   → ResolvedCommon        │
//!                │   + feature-specific map  │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │  *DraftDto              │
//!                └────────────────────────┘
//! ```
//!
//! Submodules:
//! * [`context`] — request-scoped [`ExtractContext`] (catalogs + locale + today).
//! * [`mapping`] — shared mappers (`apply_common`, `resolve_amount`, `resolve_date`).
//! * [`raw`] — [`AiCommonFields`] flattened into every `Ai*Raw` struct.
//! * [`subscription`], [`expense`], [`receipt`], [`statement`] — feature
//!   commands. Each one is a thin shell: prompt + LLM call + map.

pub mod context;
pub mod expense;
pub mod mapping;
pub mod raw;
pub mod receipt;
pub mod statement;
pub mod subscription;

pub use expense::ai_extract_expense_from_text;
pub use receipt::ai_extract_receipt;
pub use statement::ai_import_statement_file;
pub use subscription::ai_extract_subscription_from_text;
