//! AI-integration module: thin layer over the [`aisdk`] crate.
//!
//! Architecture mirrors `commands/rates/`: a small registry of providers
//! (`providers/mod.rs`) plus per-feature extraction modules under
//! [`extract`]. The extractors share an [`extract::ExtractContext`] +
//! [`extract::mapping`] / [`extract::raw`] foundation so each feature file
//! only carries its own prompt + map logic.
//!
//! Storage layout — see [`providers::storage_keys`]:
//! * `config:aiProvider`           — selected provider type (`openrouter` | `openai_compat`).
//! * `config:aiModel`              — model identifier as a string.
//! * `config:aiCustomEndpoint`     — base URL for `openai_compat` (Ollama / LM Studio / proxy).
//! * `config:aiFeatures`           — JSON object with per-feature toggles.
//! * `secure_storage.aiApiKey.<provider>` — namespaced API key (one per provider).

pub mod catalog_snapshot;
pub mod dto;
pub mod extract;
pub mod heuristics;
pub mod json_parse;
pub mod prompts;
pub mod providers;
pub mod shared;

pub use extract::ai_smart_input;
pub use providers::{ai_get_providers, ai_test_connection};
