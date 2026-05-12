//! AI-driven extraction pipeline.
//!
//! The frontend has exactly one entry point — `ai_smart_input` — which
//! accepts a `surface` ("expense" | "subscription") and *either* free-form
//! text or a file/image. The command picks the right route internally:
//!
//! ```text
//!   ┌──────────────────────────────────────────────────┐
//!   │   surface + (text | file bytes) + mime + locale  │
//!   └────────────────────────┬─────────────────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ ExtractContext::from_state │
//!                │  (catalogs + locale + today) │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │  prompts/smart.rs        │
//!                │  build(surface, kind, ctx) │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ run_text / call_vision /  │
//!                │ heuristics + chunk LLM    │
//!                └───────────┬────────────┘
//!                            │
//!                ┌───────────▼────────────┐
//!                │ AiSmartResultDto         │
//!                │ (tagged by `surface`)    │
//!                └────────────────────────┘
//! ```
//!
//! Submodules:
//! * [`context`] — request-scoped [`ExtractContext`] (catalogs + locale + today).
//! * [`mapping`] — shared mappers (`apply_common`, `resolve_amount`, …).
//! * [`raw`] — [`AiCommonFields`] flattened into LLM-response structs.
//! * [`vision_io`] — vision endpoint POST + HEIC transcoding, shared by
//!   both surfaces.
//! * [`smart`] — the single `ai_smart_input` command with 4-way dispatch.

pub mod context;
pub mod mapping;
pub mod raw;
pub mod smart;
pub mod vision_io;

pub use smart::ai_smart_input;
