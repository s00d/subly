//! Common LLM-response fields shared between extractors.
//!
//! Every extractor (subscription / expense / receipt / statement-row) returns
//! a JSON object that includes the same "what is this" header plus
//! feature-specific extras. We split out the common header here so each
//! `AiXRaw` can `#[serde(flatten)]` it and only declare the feature-specific
//! fields.

use serde::Deserialize;

/// Fields that **every** extractor asks the model for. Feature-specific
/// payload structs flatten this and add their own extras (`price`, `amount`,
/// `date`, …).
///
/// All fields are optional so a sparse model reply still deserialises — the
/// downstream mapper turns missing values into warnings, not hard errors.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCommonFields {
    /// Merchant / service / item name.
    #[serde(default)]
    pub name: String,
    /// ISO-4217 currency hint (e.g. "USD") or symbol (e.g. "₽") — resolved
    /// against [`super::super::dto::CatalogSnapshot`] later.
    #[serde(default)]
    pub currency_code: Option<String>,
    /// Free-form category hint matched against `categories[].name` /
    /// `categories[].i18n_key`.
    #[serde(default)]
    pub category_hint: Option<String>,
    /// Free-form payment method hint ("Card", "Apple Pay", …).
    #[serde(default)]
    pub payment_method_hint: Option<String>,
    /// Short note the model thinks is relevant.
    #[serde(default)]
    pub notes: Option<String>,
    /// Model's self-reported confidence in `[0, 1]`. 0 = "did my best".
    #[serde(default)]
    pub confidence: Option<f32>,
}
