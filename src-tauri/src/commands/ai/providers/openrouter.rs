//! OpenRouter provider descriptor.
//!
//! OpenRouter is a single key that fans out to 200+ models (OpenAI / Anthropic
//! / Google / Mistral / DeepSeek / Llama / …). It speaks the OpenAI Chat
//! Completions protocol, so we drive it through aisdk's `OpenAICompatible`
//! with a fixed base URL.

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "openrouter",
        name: "OpenRouter",
        default_model: "openai/gpt-4o-mini",
        default_base_url: Some("https://openrouter.ai/api/v1"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://openrouter.ai/keys",
        description: "Unified gateway to 200+ models (OpenAI, Anthropic, Google, Mistral…). One key, pay-per-token.",
        recommended_models: super::presets::OPENROUTER,
    }
}
