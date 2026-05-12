//! Recommended model presets per provider.
//!
//! Each preset gives the UI a curated dropdown of known-good models with
//! their capability flags, instead of forcing the user to type model
//! identifiers by hand. The `Custom…` entry in the UI is still available
//! for users who want to point at a specific Ollama tag.
//!
//! The list is intentionally short — extensive enumeration belongs on the
//! provider's own dashboard, not in our settings UI.

/// Compile-time preset entry. Cloned into [`super::ModelPresetDto`] for IPC.
#[derive(Debug, Clone, Copy)]
pub struct ModelPreset {
    pub id: &'static str,
    pub label: &'static str,
    /// `true` if the model accepts `image_url` content blocks (vision).
    pub supports_vision: bool,
    /// Features this preset is recommended for. Used to power
    /// "best-for-receipt" badges in the UI.
    pub recommended_for: &'static [&'static str],
}

/// IPC-friendly mirror of [`ModelPreset`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPresetDto {
    pub id: String,
    pub label: String,
    pub supports_vision: bool,
    pub recommended_for: Vec<String>,
}

impl From<&ModelPreset> for ModelPresetDto {
    fn from(preset: &ModelPreset) -> Self {
        Self {
            id: preset.id.to_string(),
            label: preset.label.to_string(),
            supports_vision: preset.supports_vision,
            recommended_for: preset.recommended_for.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// OpenRouter presets (curated subset of the 200+ catalog).
pub const OPENROUTER: &[ModelPreset] = &[
    ModelPreset {
        id: "openai/gpt-4o-mini",
        label: "GPT-4o mini · fast & cheap",
        supports_vision: true,
        recommended_for: &["subscription", "expense", "receipt", "statement"],
    },
    ModelPreset {
        id: "openai/gpt-4o",
        label: "GPT-4o · best quality",
        supports_vision: true,
        recommended_for: &["receipt", "statement"],
    },
    ModelPreset {
        id: "anthropic/claude-3.5-sonnet",
        label: "Claude 3.5 Sonnet · strong reasoning",
        supports_vision: true,
        recommended_for: &["receipt", "statement"],
    },
    ModelPreset {
        id: "google/gemini-2.0-flash-001",
        label: "Gemini 2.0 Flash · cheap multimodal",
        supports_vision: true,
        recommended_for: &["receipt", "expense", "subscription"],
    },
    ModelPreset {
        id: "meta-llama/llama-3.3-70b-instruct",
        label: "Llama 3.3 70B · open weights",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
    ModelPreset {
        id: "deepseek/deepseek-chat-v3",
        label: "DeepSeek Chat v3 · low-cost",
        supports_vision: false,
        recommended_for: &["subscription", "expense", "statement"],
    },
];

/// OpenAI (direct) presets.
pub const OPENAI: &[ModelPreset] = &[
    ModelPreset {
        id: "gpt-4o-mini",
        label: "GPT-4o mini · fast & cheap (vision)",
        supports_vision: true,
        recommended_for: &["subscription", "expense", "receipt", "statement"],
    },
    ModelPreset {
        id: "gpt-4o",
        label: "GPT-4o · best quality (vision)",
        supports_vision: true,
        recommended_for: &["receipt", "statement"],
    },
    ModelPreset {
        id: "gpt-4-turbo",
        label: "GPT-4 Turbo (vision)",
        supports_vision: true,
        recommended_for: &["receipt", "statement"],
    },
    ModelPreset {
        id: "gpt-3.5-turbo",
        label: "GPT-3.5 Turbo · cheap, text-only",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
];

/// Groq presets — open-weight LLMs on LPU silicon.
pub const GROQ: &[ModelPreset] = &[
    ModelPreset {
        id: "llama-3.3-70b-versatile",
        label: "Llama 3.3 70B · default",
        supports_vision: false,
        recommended_for: &["subscription", "expense", "statement"],
    },
    ModelPreset {
        id: "llama-3.1-8b-instant",
        label: "Llama 3.1 8B · instant",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
    ModelPreset {
        id: "mixtral-8x7b-32768",
        label: "Mixtral 8x7B · long context",
        supports_vision: false,
        recommended_for: &["statement"],
    },
];

/// DeepSeek presets.
pub const DEEPSEEK: &[ModelPreset] = &[
    ModelPreset {
        id: "deepseek-chat",
        label: "DeepSeek Chat · low-cost",
        supports_vision: false,
        recommended_for: &["subscription", "expense", "statement"],
    },
    ModelPreset {
        id: "deepseek-reasoner",
        label: "DeepSeek Reasoner · stronger",
        supports_vision: false,
        recommended_for: &["statement"],
    },
];

/// Google Gemini presets (via OpenAI-compat endpoint). All multimodal.
pub const GEMINI: &[ModelPreset] = &[
    ModelPreset {
        id: "gemini-2.0-flash",
        label: "Gemini 2.0 Flash · cheap & fast (vision)",
        supports_vision: true,
        recommended_for: &["subscription", "expense", "receipt", "statement"],
    },
    ModelPreset {
        id: "gemini-1.5-pro",
        label: "Gemini 1.5 Pro · best quality (vision)",
        supports_vision: true,
        recommended_for: &["receipt", "statement"],
    },
    ModelPreset {
        id: "gemini-1.5-flash",
        label: "Gemini 1.5 Flash · fast (vision)",
        supports_vision: true,
        recommended_for: &["subscription", "expense", "receipt"],
    },
];

/// Mistral AI presets.
pub const MISTRAL: &[ModelPreset] = &[
    ModelPreset {
        id: "mistral-large-latest",
        label: "Mistral Large · best quality",
        supports_vision: false,
        recommended_for: &["statement", "expense"],
    },
    ModelPreset {
        id: "mistral-small-latest",
        label: "Mistral Small · default",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
    ModelPreset {
        id: "codestral-latest",
        label: "Codestral · structured text",
        supports_vision: false,
        recommended_for: &["statement"],
    },
    ModelPreset {
        id: "pixtral-12b-2409",
        label: "Pixtral 12B · open-source vision",
        supports_vision: true,
        recommended_for: &["receipt"],
    },
];

/// Self-hosted / OpenAI-compatible presets (Ollama, LM Studio).
pub const OPENAI_COMPAT: &[ModelPreset] = &[
    ModelPreset {
        id: "llama3.1:8b",
        label: "Llama 3.1 8B · default Ollama",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
    ModelPreset {
        id: "qwen2.5:7b",
        label: "Qwen 2.5 7B · multilingual",
        supports_vision: false,
        recommended_for: &["subscription", "expense", "statement"],
    },
    ModelPreset {
        id: "mistral:7b",
        label: "Mistral 7B · fast",
        supports_vision: false,
        recommended_for: &["subscription", "expense"],
    },
    ModelPreset {
        id: "llava:13b",
        label: "LLaVA 13B · open-source vision",
        supports_vision: true,
        recommended_for: &["receipt"],
    },
    ModelPreset {
        id: "llama3.2-vision:11b",
        label: "Llama 3.2 Vision 11B · multimodal",
        supports_vision: true,
        recommended_for: &["receipt", "expense"],
    },
];

/// Lookup helper — returns the preset for a given `provider_type` + model id,
/// or `None` if the user typed a custom model.
///
/// Consumed by Phase 4's backend feature guard (`require_vision_capable`).
#[allow(dead_code)]
pub fn find(provider_type: &str, model_id: &str) -> Option<&'static ModelPreset> {
    let presets: &[ModelPreset] = match provider_type {
        "openrouter" => OPENROUTER,
        "openai_compat" => OPENAI_COMPAT,
        "openai" => OPENAI,
        "groq" => GROQ,
        "deepseek" => DEEPSEEK,
        "gemini" => GEMINI,
        "mistral" => MISTRAL,
        _ => return None,
    };
    presets.iter().find(|p| p.id.eq_ignore_ascii_case(model_id))
}

/// Whether the active provider + model combo is known to support vision.
///
/// `false` for custom models (we can't verify), `true` for matching presets
/// with `supports_vision: true`.
#[allow(dead_code)]
pub fn vision_supported(provider_type: &str, model_id: &str) -> bool {
    find(provider_type, model_id)
        .map(|p| p.supports_vision)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::super::{provider_descriptors, AiProviderDescriptor};
    use super::*;

    fn descriptor(provider_type: &str) -> AiProviderDescriptor {
        provider_descriptors()
            .into_iter()
            .find(|d| d.provider_type == provider_type)
            .unwrap_or_else(|| panic!("provider {provider_type} not registered"))
    }

    #[test]
    fn every_provider_has_recommended_models() {
        for d in provider_descriptors() {
            assert!(
                !d.recommended_models.is_empty(),
                "provider {} should expose at least one recommended model",
                d.provider_type,
            );
        }
    }

    #[test]
    fn presets_recommend_for_known_feature_kinds() {
        const KNOWN: &[&str] = &["subscription", "expense", "receipt", "statement"];
        for d in provider_descriptors() {
            for preset in d.recommended_models.iter() {
                for kind in preset.recommended_for.iter() {
                    assert!(
                        KNOWN.contains(kind),
                        "{} preset {} has unknown recommended_for {:?}",
                        d.provider_type,
                        preset.id,
                        kind,
                    );
                }
            }
        }
    }

    #[test]
    fn openai_descriptor_uses_correct_endpoint() {
        let d = descriptor("openai");
        assert_eq!(d.default_base_url, Some("https://api.openai.com/v1"));
        assert!(d.requires_key);
        assert!(!d.requires_endpoint);
        // GPT-4o family should be present.
        assert!(d.recommended_models.iter().any(|p| p.id == "gpt-4o"));
    }

    #[test]
    fn groq_descriptor_is_text_only() {
        let d = descriptor("groq");
        assert_eq!(d.default_base_url, Some("https://api.groq.com/openai/v1"));
        assert!(d.recommended_models.iter().all(|p| !p.supports_vision));
    }

    #[test]
    fn deepseek_descriptor_is_text_only() {
        let d = descriptor("deepseek");
        assert_eq!(d.default_base_url, Some("https://api.deepseek.com/v1"));
        assert!(d.recommended_models.iter().all(|p| !p.supports_vision));
    }

    #[test]
    fn gemini_descriptor_supports_vision() {
        let d = descriptor("gemini");
        assert_eq!(
            d.default_base_url,
            Some("https://generativelanguage.googleapis.com/v1beta/openai"),
        );
        assert!(d.recommended_models.iter().all(|p| p.supports_vision));
    }

    #[test]
    fn mistral_descriptor_has_pixtral_for_vision() {
        let d = descriptor("mistral");
        assert_eq!(d.default_base_url, Some("https://api.mistral.ai/v1"));
        let pixtral = d
            .recommended_models
            .iter()
            .find(|p| p.id == "pixtral-12b-2409")
            .expect("pixtral should be present");
        assert!(pixtral.supports_vision);
    }

    #[test]
    fn ollama_docs_url_is_fresh() {
        let d = descriptor("openai_compat");
        assert_eq!(d.docs_url, "https://docs.ollama.com/openai");
    }

    #[test]
    fn find_resolves_each_new_provider() {
        assert!(find("openai", "gpt-4o-mini").is_some());
        assert!(find("groq", "llama-3.3-70b-versatile").is_some());
        assert!(find("deepseek", "deepseek-chat").is_some());
        assert!(find("gemini", "gemini-2.0-flash").is_some());
        assert!(find("mistral", "pixtral-12b-2409").is_some());
        // Unknown provider/model returns None.
        assert!(find("nonexistent", "anything").is_none());
        assert!(find("openai", "no-such-model").is_none());
    }

    #[test]
    fn vision_supported_matches_preset_flag() {
        assert!(vision_supported("openai", "gpt-4o"));
        assert!(!vision_supported("groq", "llama-3.1-8b-instant"));
        assert!(!vision_supported("openai", "custom-model")); // not a preset
    }
}
