//! OpenAI provider descriptor (direct, not via OpenRouter).
//!
//! OpenAI speaks the OpenAI Chat Completions protocol natively, so we drive
//! it through aisdk's `OpenAICompatible` with `api.openai.com` as the base
//! URL. The same provider also handles vision via `/chat/completions` with
//! `image_url` content blocks — already supported by [`super::super::extract::receipt`].

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "openai",
        name: "OpenAI",
        default_model: "gpt-4o-mini",
        default_base_url: Some("https://api.openai.com/v1"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://platform.openai.com/api-keys",
        description: "Direct OpenAI API. GPT-4o (vision), GPT-4o mini (fast/cheap), GPT-4 Turbo, GPT-3.5 Turbo.",
        recommended_models: super::presets::OPENAI,
    }
}
