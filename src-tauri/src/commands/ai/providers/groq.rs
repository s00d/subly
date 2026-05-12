//! Groq provider descriptor.
//!
//! Groq runs open-weight models (Llama 3.x, Mixtral) on custom LPU silicon
//! with extremely low latency. The Chat Completions API at
//! `api.groq.com/openai/v1` is a drop-in OpenAI-compatible endpoint.
//! Vision is not supported (text-only LLMs).

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "groq",
        name: "Groq",
        default_model: "llama-3.3-70b-versatile",
        default_base_url: Some("https://api.groq.com/openai/v1"),
        requires_endpoint: false,
        requires_key: true,
        docs_url: "https://console.groq.com/keys",
        description: "Ultra-fast inference for open-weight models (Llama, Mixtral). Generous free tier; text-only.",
        recommended_models: super::presets::GROQ,
    }
}
