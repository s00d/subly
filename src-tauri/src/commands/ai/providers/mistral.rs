//! Mistral AI provider descriptor.
//!
//! Mistral's `api.mistral.ai/v1` is OpenAI-compatible. Pixtral models add
//! native vision support; the rest of the family (Large, Small, Codestral)
//! is text-only.

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "mistral",
        name: "Mistral AI",
        default_model: "mistral-small-latest",
        default_base_url: Some("https://api.mistral.ai/v1"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://console.mistral.ai/api-keys",
        description: "Mistral models (Large, Small, Codestral). Pixtral adds vision support. Strong for European languages.",
        recommended_models: super::presets::MISTRAL,
    }
}
