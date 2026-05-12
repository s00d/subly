//! Statement-extractor system prompt builder.
//!
//! Accepts the [`HeuristicResult`] so we can echo the detected source
//! columns back to the model — that single hint cuts hallucination rates
//! noticeably on tabular CSV data.

use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::heuristics::HeuristicResult;
use crate::commands::ai::prompts::fewshot::{block, FeatureKind};
use crate::commands::ai::prompts::fragments::{
    category_rules, currency_rules, date_rules, json_reply_rule, language_rule, precision_rule,
    system_header,
};

pub fn build(ctx: &ExtractContext, heur: &HeuristicResult) -> String {
    let currencies = ctx.catalogs.render_currency_codes();
    let categories = ctx.catalogs.render_category_names();
    let language = language_rule(ctx.locale.as_deref());
    let examples = block(FeatureKind::Statement, &ctx.today);
    let header_hint = if !heur.detected_columns.header_row.is_empty() {
        format!(
            "\nDetected source columns: {}.",
            heur.detected_columns.header_row.join(" | "),
        )
    } else {
        String::new()
    };

    format!(
        r#"{header}
You receive ONE OR MORE raw lines from a bank statement (one transaction per
line, columns separated by whitespace, comma, or pipe). Convert them into
structured JSON.

{reply_rule}

Output schema:
{{
  "transactions": [
    {{
      "name":              string  // merchant / description
      "amount":            number  // positive number (we consider every line a spend)
      "currencyCode":      string  // ISO-4217 if you can infer it
      "date":              string  // YYYY-MM-DD; "today" = {today}
      "categoryHint":      string  // optional
      "paymentMethodHint": string  // optional
      "notes":             string  // optional short note
      "confidence":        number  // 0..1
    }}
  ]
}}

Statement-specific:
- Skip rows that are clearly NOT transactions (headers, totals, summaries).
- Negative amounts → take the absolute value (we already know they're expenses).
- Each line maps to at most ONE transaction (do not split bundled items).{header_hint}

{precision}

{currency}

{category}

{dates}
{language}
{examples}"#,
        header = system_header("bank-statement"),
        reply_rule = json_reply_rule(),
        today = ctx.today,
        precision = precision_rule(),
        currency = currency_rules(&currencies),
        category = category_rules(&categories),
        dates = date_rules(&ctx.today),
        language = language,
        examples = examples,
        header_hint = header_hint,
    )
}
