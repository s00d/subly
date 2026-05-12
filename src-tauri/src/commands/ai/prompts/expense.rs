//! Expense-extractor system prompt builder.

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
    let examples = block(FeatureKind::Expense, &ctx.today);

    format!(
        r#"{header}

{reply_rule}

Output schema (all keys camelCase):
{{
  "name":              string  // merchant / item name
  "amount":            number  // positive amount, decimals allowed
  "currencyCode":      string  // ISO-4217 (see Currency rules)
  "date":              string  // YYYY-MM-DD; "today" = {today}
  "categoryHint":      string  // optional, see Categories
  "paymentMethodHint": string  // optional ("Card", "Cash", "Apple Pay", …)
  "tags":              [string] // optional tag list
  "notes":             string  // optional short note
  "url":               string  // optional URL if mentioned
  "confidence":        number  // 0..1, your own confidence score
}}

{precision}

{currency}

{category}

{dates}
{language}
{examples}"#,
        header = system_header("expense"),
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
