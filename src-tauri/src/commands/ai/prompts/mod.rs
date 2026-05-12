//! Centralised system prompts for AI features.
//!
//! Layout:
//! * Per-feature builder modules (`subscription`, `expense`, `receipt`,
//!   `statement`) — each exposes a `pub fn build(ctx) -> String` that
//!   assembles the final system prompt.
//! * [`fragments`] — small reusable blocks (header, JSON-reply rule,
//!   currency rules, category rules, date rules, **language rule**).
//! * [`fewshot`] — short Input → JSON exemplars per feature, including
//!   Russian examples so models behave well on RU input.
//!
//! Phase 1 ships the file layout with the old prompts ported 1:1.
//! Phase 2 enriches the fragments (language_rule with explicit `Reply in
//! <Language>`, few-shot examples, etc.).

pub mod expense;
pub mod fewshot;
pub mod fragments;
pub mod receipt;
pub mod statement;
pub mod subscription;

/// Connectivity probe used by `ai_test_connection`.
///
/// Kept intentionally tiny so it works on free-tier OpenRouter models and
/// rate-limited local Ollama installs.
pub const TEST_SYSTEM: &str =
    "You are a connectivity probe. Reply with exactly the single word 'pong' and nothing else.";

pub const TEST_PROMPT: &str = "ping";
