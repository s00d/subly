//! Google Gemini provider descriptor (via OpenAI-compatible endpoint).
//!
//! Google exposes Gemini through an OpenAI-compatible REST API at
//! `generativelanguage.googleapis.com/v1beta/openai`. All flagship
//! Gemini models accept image inputs, so vision is enabled for receipt
//! parsing.
//!
//! API key: any Google AI Studio key (`https://aistudio.google.com/apikey`).

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "gemini",
        name: "Google Gemini",
        default_model: "gemini-2.0-flash",
        default_base_url: Some("https://generativelanguage.googleapis.com/v1beta/openai"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://aistudio.google.com/apikey",
        description: "Google's Gemini through an OpenAI-compatible endpoint. Generous free tier, vision support across the family.",
        recommended_models: super::presets::GEMINI,
    }
}
