//! DeepSeek provider descriptor.
//!
//! DeepSeek API speaks OpenAI Chat Completions natively at `api.deepseek.com/v1`.
//! Models are text-only (no vision). The reasoning model `deepseek-reasoner`
//! is slower but stronger at structured extraction.

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "deepseek",
        name: "DeepSeek",
        default_model: "deepseek-chat",
        default_base_url: Some("https://api.deepseek.com/v1"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://platform.deepseek.com/api_keys",
        description: "Very low-cost OpenAI-compatible API. `deepseek-chat` for general use, `deepseek-reasoner` for harder extractions. Text-only.",
        recommended_models: super::presets::DEEPSEEK,
    }
}
