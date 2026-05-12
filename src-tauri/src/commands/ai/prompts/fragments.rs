//! Reusable prompt fragments.
//!
//! Each helper returns a small block of text that feature prompts splice
//! together. Keeping them in one place ensures every extractor speaks the
//! same dialect (currency rules, date rules, language rule, JSON-reply
//! rule…) so improving one fragment improves every prompt.

/// Map BCP-47 short codes to a human-readable English name. Models follow
/// instructions like *"Reply in Russian"* far more reliably than
/// *"Reply in ru"*, so we look up the name before inlining it into the
/// prompt.
const LOCALE_LABEL: &[(&str, &str)] = &[
    ("en", "English"),
    ("ru", "Russian"),
    ("de", "German"),
    ("fr", "French"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh", "Chinese"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("tr", "Turkish"),
];

/// Look up the English name for a BCP-47 short code, falling back to the
/// raw code wrapped in quotes so the LLM still has *some* hint.
pub fn locale_name(code: &str) -> String {
    let normalized = code.trim().to_lowercase();
    let short = normalized.split('-').next().unwrap_or(&normalized);
    LOCALE_LABEL
        .iter()
        .find(|(c, _)| *c == short)
        .map(|(_, name)| (*name).to_string())
        .unwrap_or_else(|| format!("'{short}'"))
}

/// Single canonical "USER_LANGUAGE" block. We hammer the rule in three
/// times (language for free-form fields, ISO/English for keys, dates in
/// ISO) because GPT-class models routinely drift back into English for
/// `name` when the directive is mentioned only once.
pub fn language_rule(locale: Option<&str>) -> String {
    let Some(code) = locale.map(|c| c.trim()).filter(|c| !c.is_empty()) else {
        // No locale → assume English; keep things explicit anyway.
        return "\nUSER_LANGUAGE: English\nFree-form fields (name, notes) MUST be written in English.".to_string();
    };
    let name = locale_name(code);
    format!(
        "\nUSER_LANGUAGE: {name}\n\
         Free-form fields (name, notes) MUST be written in {name}.\n\
         However, JSON keys, `currencyCode` (ISO-4217) and dates (YYYY-MM-DD) \
         MUST stay in English/ISO regardless of user language."
    )
}

/// Generic "you are X" opener. Keeping the persona terse leaves more
/// tokens for the schema + few-shot block.
pub fn system_header(feature: &str) -> String {
    format!("You are Subly's {feature} extractor. Be precise, conservative, and never invent data.")
}

/// JSON-only reply discipline. Every extractor prepends this so the
/// downstream `json_parse::parse_llm_json` rarely has to strip prose.
pub fn json_reply_rule() -> &'static str {
    "Reply with ONLY a single JSON object — no markdown fences, no commentary, no leading text."
}

/// Constrain `currencyCode` to one of the user's known currencies. The list
/// is sorted + deduped by [`CatalogSnapshot::render_currency_codes`].
pub fn currency_rules(currencies: &str) -> String {
    format!(
        "Currency:\n\
         - `currencyCode` MUST be an ISO-4217 code from: {currencies}.\n\
         - Common mappings: \"$\" / \"USD\" → \"USD\"; \"€\" / \"EUR\" → \"EUR\"; \
         \"₽\" / \"руб\" / \"р.\" / \"rub\" → \"RUB\"; \"£\" / \"GBP\" → \"GBP\".\n\
         - If you cannot tell, omit the field (do NOT guess)."
    )
}

/// Constrain `categoryHint` to one of the user's existing categories.
pub fn category_rules(categories: &str) -> String {
    format!(
        "Categories:\n\
         - Allowed values for `categoryHint`: {categories}.\n\
         - Pick the closest match. If nothing fits cleanly, omit `categoryHint`."
    )
}

/// Date normalisation rules anchored on `today` (YYYY-MM-DD).
pub fn date_rules(today: &str) -> String {
    format!(
        "Dates:\n\
         - Always reply with `YYYY-MM-DD`.\n\
         - Today is {today}. Resolve relative phrases against it (\"today\", \
         \"сегодня\", \"heute\" → {today}; \"yesterday\" / \"вчера\" / \"gestern\" \
         → {today} minus one day).\n\
         - If the input is ambiguous (e.g. \"03/04/2026\"), prefer day-first \
         interpretation; if you can't tell, omit the field."
    )
}

/// "Don't invent data" rule kept consistent across extractors. Goes after
/// the schema so the model reads the schema first, then the discipline.
pub fn precision_rule() -> &'static str {
    "Precision:\n\
     - If a field is unknown, omit it or set it to `\"\"` / `null`. NEVER invent values.\n\
     - Negative bank amounts are still expenses — take the absolute value.\n\
     - Prefer integers when the input has no decimal part (\"300 rub\" → 300, not 300.00)."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_name_known_codes() {
        assert_eq!(locale_name("ru"), "Russian");
        assert_eq!(locale_name("en"), "English");
        assert_eq!(locale_name("RU"), "Russian"); // case-insensitive
        assert_eq!(locale_name("ru-RU"), "Russian"); // strips BCP-47 region
        assert_eq!(locale_name("zh"), "Chinese");
    }

    #[test]
    fn locale_name_unknown_quoted() {
        assert_eq!(locale_name("xx"), "'xx'");
        assert_eq!(locale_name(""), "''");
    }

    #[test]
    fn language_rule_includes_name() {
        let out = language_rule(Some("ru"));
        assert!(out.contains("USER_LANGUAGE: Russian"));
        assert!(out.contains("MUST be written in Russian"));
        assert!(out.contains("ISO-4217"));
    }

    #[test]
    fn language_rule_defaults_to_english_for_none() {
        let out = language_rule(None);
        assert!(out.contains("English"));
    }
}
