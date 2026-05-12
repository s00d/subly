//! Subscription-extractor system prompt builder.
//!
//! Composes reusable [`super::fragments`] blocks plus a feature-specific
//! schema. The body stays in English so the model has a single canonical
//! reference; the user's language is enforced by the embedded
//! `language_rule` block from `fragments`.

use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::prompts::fewshot::{block, FeatureKind};
use crate::commands::ai::prompts::fragments::{
    category_rules, currency_rules, language_rule, precision_rule, system_header,
    json_reply_rule,
};

pub fn build(ctx: &ExtractContext) -> String {
    let currencies = ctx.catalogs.render_currency_codes();
    let categories = ctx.catalogs.render_category_names();
    let language = language_rule(ctx.locale.as_deref());
    let examples = block(FeatureKind::Subscription, &ctx.today);

    format!(
        r#"{header}

{reply_rule}

Output schema (all keys camelCase):
{{
  "name":              string  // service / product name, e.g. "Telegram Premium"
  "price":             number  // periodic price (positive number, decimals allowed)
  "currencyCode":      string  // ISO-4217 (see Currency rules)
  "cycle":             "day" | "week" | "month" | "year"
  "frequency":         integer // billing every N cycles, default 1
  "categoryHint":      string  // optional, see Categories
  "paymentMethodHint": string  // optional ("Card", "Apple Pay", "Bank transfer", …)
  "startDate":         string  // optional YYYY-MM-DD
  "nextPayment":       string  // optional YYYY-MM-DD
  "notes":             string  // optional short note
  "url":               string  // optional URL if mentioned
  "tags":              [string] // optional tag list
  "confidence":        number  // 0..1, your own confidence score
}}

{precision}

{currency}

{category}

Cadence:
- "monthly" / "per month" / "/мес" → cycle "month", frequency 1.
- "yearly" / "annual" / "в год" → cycle "year", frequency 1.
- "every N months" / "раз в N месяцев" → cycle "month", frequency N.
{language}
{examples}"#,
        header = system_header("subscription"),
        reply_rule = json_reply_rule(),
        precision = precision_rule(),
        currency = currency_rules(&currencies),
        category = category_rules(&categories),
        language = language,
        examples = examples,
    )
}
