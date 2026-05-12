//! Generic OpenAI-compatible endpoint descriptor.
//!
//! Covers self-hosted runtimes that expose `/v1/chat/completions`
//! (Ollama, LM Studio, vLLM, Text Generation WebUI…), corporate proxies
//! and any third-party gateway that wasn't worth a dedicated descriptor.

pub fn descriptor() -> super::AiProviderDescriptor {
    super::AiProviderDescriptor {
        provider_type: "openai_compat",
        name: "Custom OpenAI-compatible",
        default_model: "llama3.1:8b",
        default_base_url: None,
        requires_endpoint: true,
        requires_key: false,
        docs_url: "https://docs.ollama.com/openai",
        description: "Bring-your-own endpoint: Ollama, LM Studio, vLLM, corporate gateways. Fully offline if desired.",
        recommended_models: super::presets::OPENAI_COMPAT,
    }
}
