//! Receipt-extractor system prompt builder.
//!
//! Used by both vision-LLM (image bytes) and text path (PDF with embedded
//! text). The schema is identical; only the user-prompt differs.

use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::prompts::fewshot::{block, FeatureKind};
use crate::commands::ai::prompts::fragments::{
    category_rules, currency_rules, date_rules, json_reply_rule, language_rule, precision_rule,
    system_header,
};

pub fn build(ctx: &ExtractContext) -> String {
    let currencies = ctx.catalogs.render_currency_codes();
    let categories = ctx.catalogs.render_category_names();
    let language = language_rule(ctx.locale.as_deref());
    let examples = block(FeatureKind::Receipt, &ctx.today);

    format!(
        r#"{header}

{reply_rule}

Output schema (all keys camelCase):
{{
  "name":              string  // merchant / store name
  "amount":            number  // GRAND TOTAL (positive number)
  "currencyCode":      string  // ISO-4217 (see Currency rules)
  "date":              string  // YYYY-MM-DD; if unclear use {today}
  "categoryHint":      string  // optional, see Categories
  "paymentMethodHint": string  // optional ("Card", "Cash", "Apple Pay", …)
  "notes":             string  // optional short note
  "lineItems":         [       // optional itemised lines
    {{ "name": string, "amount": number }}
  ],
  "confidence":        number  // 0..1
}}

Receipt-specific:
- Use the GRAND TOTAL as `amount`. Ignore subtotals and tax-only lines.
- If the receipt has 3+ purchased items, populate `lineItems` (max 10 lines).

{precision}

{currency}

{category}

{dates}
{language}
{examples}"#,
        header = system_header("receipt"),
        reply_rule = json_reply_rule(),
        today = ctx.today,
        precision = precision_rule(),
        currency = currency_rules(&currencies),
        category = category_rules(&categories),
        dates = date_rules(&ctx.today),
        language = language,
        examples = examples,
    )
}
