//! Shared mapping helpers for AI extractors.
//!
//! Every `extract_*` command does the same chain after the LLM response:
//! 1. Resolve `currencyCode` → internal `currency_id` (with warning if unknown).
//! 2. Resolve `categoryHint` → internal `category_id`.
//! 3. Resolve `paymentMethodHint` → internal `payment_method_id`.
//! 4. Normalise dates to `YYYY-MM-DD`.
//! 5. Clamp confidence to `[0, 1]`.
//!
//! Pulling those out cuts each `extract_*.rs` to ~80-100 lines.

use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::raw::AiCommonFields;

/// Output of [`apply_common`]: everything the LLM gave us, mapped to internal
/// IDs and accumulated warnings. Mappers in `extract/<feature>.rs` extend the
/// warning vec with their own field-specific checks (price/amount/date).
pub struct ResolvedCommon {
    pub name: String,
    pub currency_id: String,
    /// Original (uppercased, trimmed) currency code echoed back by the LLM.
    /// Kept around so the UI can show "we guessed RUB" hints.
    pub currency_code: String,
    pub category_id: String,
    pub category_hint: String,
    pub payment_method_id: String,
    pub notes: String,
    pub confidence: f32,
    pub warnings: Vec<String>,
}

/// Map [`AiCommonFields`] to internal IDs + warnings against the live catalog
/// snapshot. The output is the starting point for every feature-specific
/// mapper.
pub fn apply_common(raw: &AiCommonFields, ctx: &ExtractContext) -> ResolvedCommon {
    let mut warnings: Vec<String> = Vec::new();

    let name = raw.name.trim().to_string();
    if name.is_empty() {
        warnings.push("ai_warning_name_missing".to_string());
    }

    let currency_code = raw
        .currency_code
        .clone()
        .map(|c| c.trim().to_uppercase())
        .unwrap_or_default();
    let currency_id = ctx
        .catalogs
        .resolve_currency_id(&currency_code, &ctx.main_currency_id);
    if currency_id.is_empty() {
        warnings.push("ai_warning_currency_unresolved".to_string());
    } else if !currency_code.is_empty()
        && !ctx
            .catalogs
            .currencies
            .iter()
            .any(|c| c.code.eq_ignore_ascii_case(&currency_code))
    {
        warnings.push("ai_warning_currency_substituted".to_string());
    }

    let category_hint = raw.category_hint.clone().unwrap_or_default();
    let category_id = ctx.catalogs.resolve_category_id(&category_hint);
    if !category_hint.trim().is_empty() && category_id.is_empty() {
        warnings.push("ai_warning_category_unresolved".to_string());
    }

    let payment_method_hint = raw.payment_method_hint.clone().unwrap_or_default();
    let payment_method_id = ctx.catalogs.resolve_payment_method_id(&payment_method_hint);

    ResolvedCommon {
        name,
        currency_id,
        currency_code,
        category_id,
        category_hint,
        payment_method_id,
        notes: raw.notes.clone().unwrap_or_default(),
        confidence: clamp_confidence(raw.confidence),
        warnings,
    }
}

/// Normalise an LLM-supplied amount. Negative values are made positive (banks
/// often render expenses as negative numbers); zero / missing surfaces a
/// warning instead of failing the request.
pub fn resolve_amount(opt: Option<f64>) -> (f64, Vec<String>) {
    let mut warnings = Vec::new();
    let amount = opt.unwrap_or(0.0).abs();
    if amount <= 0.0 {
        warnings.push("ai_warning_amount_missing".to_string());
    }
    (amount, warnings)
}

/// Same as [`resolve_amount`] but tailored to subscriptions ("price" can be a
/// rounded integer; never negative).
pub fn resolve_price(opt: Option<f64>) -> (f64, Vec<String>) {
    let mut warnings = Vec::new();
    let price = opt.unwrap_or(0.0).max(0.0);
    if price <= 0.0 {
        warnings.push("ai_warning_price_missing".to_string());
    }
    (price, warnings)
}

/// Parse a loose date string into `YYYY-MM-DD`. Empty / missing → `""` (no
/// warning); unparseable → `""` with an explicit warning.
pub fn resolve_date(raw: Option<&str>) -> (String, Vec<String>) {
    let mut warnings = Vec::new();
    let value = match raw {
        Some(v) => v.trim(),
        None => "",
    };
    if value.is_empty() {
        return (String::new(), warnings);
    }
    match crate::models::parse_loose_date_to_ymd(value) {
        Ok((y, m, d)) => (format!("{:04}-{:02}-{:02}", y, m, d), warnings),
        Err(_) => {
            warnings.push("ai_warning_date_unresolved".to_string());
            (String::new(), warnings)
        }
    }
}

/// Like [`resolve_date`] but for optional fields (e.g. `startDate` on a
/// subscription). Returns `None` if the input is empty or unparseable — no
/// warning, the UI just won't pre-fill that field.
pub fn resolve_optional_date(raw: Option<&str>) -> Option<String> {
    let value = raw?.trim();
    if value.is_empty() {
        return None;
    }
    crate::models::parse_loose_date_to_ymd(value)
        .ok()
        .map(|(y, m, d)| format!("{:04}-{:02}-{:02}", y, m, d))
}

/// Clamp `confidence` to `[0, 1]`, defaulting to `0.0` for missing values.
pub fn clamp_confidence(raw: Option<f32>) -> f32 {
    raw.unwrap_or(0.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_positive_passes_through() {
        let (a, w) = resolve_amount(Some(12.50));
        assert!((a - 12.50).abs() < 1e-9);
        assert!(w.is_empty());
    }

    #[test]
    fn amount_negative_is_flipped() {
        let (a, w) = resolve_amount(Some(-50.0));
        assert!((a - 50.0).abs() < 1e-9);
        assert!(w.is_empty());
    }

    #[test]
    fn amount_missing_warns() {
        let (a, w) = resolve_amount(None);
        assert_eq!(a, 0.0);
        assert_eq!(w, vec!["ai_warning_amount_missing".to_string()]);

        let (a, w) = resolve_amount(Some(0.0));
        assert_eq!(a, 0.0);
        assert_eq!(w, vec!["ai_warning_amount_missing".to_string()]);
    }

    #[test]
    fn price_missing_uses_price_specific_warning() {
        let (_p, w) = resolve_price(None);
        assert_eq!(w, vec!["ai_warning_price_missing".to_string()]);
    }

    #[test]
    fn date_iso_is_normalised() {
        let (d, w) = resolve_date(Some("2026-05-11"));
        assert_eq!(d, "2026-05-11");
        assert!(w.is_empty());
    }

    #[test]
    fn date_empty_is_silent() {
        let (d, w) = resolve_date(Some(""));
        assert!(d.is_empty());
        assert!(w.is_empty());

        let (d, w) = resolve_date(None);
        assert!(d.is_empty());
        assert!(w.is_empty());
    }

    #[test]
    fn date_invalid_warns() {
        let (d, w) = resolve_date(Some("not a date"));
        assert!(d.is_empty());
        assert_eq!(w, vec!["ai_warning_date_unresolved".to_string()]);
    }

    #[test]
    fn optional_date_handles_garbage() {
        assert_eq!(resolve_optional_date(Some("2026-12-31")).as_deref(), Some("2026-12-31"));
        assert!(resolve_optional_date(Some("")).is_none());
        assert!(resolve_optional_date(None).is_none());
        assert!(resolve_optional_date(Some("xyz")).is_none());
    }

    #[test]
    fn confidence_clamps() {
        assert_eq!(clamp_confidence(Some(0.5)), 0.5);
        assert_eq!(clamp_confidence(Some(-0.1)), 0.0);
        assert_eq!(clamp_confidence(Some(1.7)), 1.0);
        assert_eq!(clamp_confidence(None), 0.0);
    }
}
