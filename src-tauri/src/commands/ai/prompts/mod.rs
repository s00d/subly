//! Centralised system prompts for AI features.
//!
//! Layout:
//! * [`smart`] — single surface-aware builder used by `ai_smart_input`.
//!   Produces an envelope-shaped prompt for any `(surface, input_kind)`
//!   pair by stitching together the shared fragments.
//! * [`statement`] — chunk-level prompt used by the heuristics fallback
//!   pipeline when we already have parsed tabular data and only need the
//!   LLM to enrich the un-resolved rows.
//! * [`fragments`] — small reusable blocks (header, JSON-reply rule,
//!   currency rules, category rules, date rules, language rule).
//! * [`fewshot`] — short Input → JSON exemplars per surface.

pub mod fewshot;
pub mod fragments;
pub mod smart;
pub mod statement;

/// Connectivity probe used by `ai_test_connection`.
///
/// Kept intentionally tiny so it works on free-tier OpenRouter models and
/// rate-limited local Ollama installs.
pub const TEST_SYSTEM: &str =
    "You are a connectivity probe. Reply with exactly the single word 'pong' and nothing else.";

pub const TEST_PROMPT: &str = "ping";
