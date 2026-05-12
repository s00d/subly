//! Unified prompt builder for `ai_smart_input`.
//!
//! One module replaces the previous four (`expense`, `subscription`,
//! `import_image`, `import_subscription`). The wire prompt stays compact
//! because we materialise only the schema block relevant to the current
//! `(surface, input_kind)` combo — but the Rust source is a single file so
//! teaching the model a new trick (say, "always echo back the merchant
//! name") is one edit.
//!
//! Output envelope is uniform:
//! ```json
//! { "kind": ..., "rows": [...], "metadata": { ... } }
//! ```
//! where `rows` are `transactions` for the expense surface and
//! `subscriptions` for the subscription surface (the model is told which
//! key to use; the Rust parser accepts both as a fallback).

use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::prompts::fewshot::{block, FeatureKind};
use crate::commands::ai::prompts::fragments::{
    category_rules, currency_rules, date_rules, json_reply_rule, language_rule,
    payment_method_rules, precision_rule, system_header, tag_rules,
};

/// What the user clicked. Decides which schema block + few-shot examples
/// get spliced into the prompt and what the resulting `kind` values mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Expense,
    Subscription,
}

/// Shape of the user input — drives the narration block telling the model
/// what to expect (free-form text vs. a picture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Text,
    Image,
}

pub fn build(ctx: &ExtractContext, surface: Surface, input_kind: InputKind) -> String {
    let currencies = ctx.catalogs.render_currency_codes();
    let categories = ctx.catalogs.render_category_names();
    let payments = ctx.catalogs.render_payment_method_names();
    let tags = ctx.catalogs.render_tag_names();
    let language = language_rule(ctx.locale.as_deref());
    let feature_label = match (surface, input_kind) {
        (Surface::Expense, InputKind::Text) => "expense-text",
        (Surface::Expense, InputKind::Image) => "expense-image",
        (Surface::Subscription, InputKind::Text) => "subscription-text",
        (Surface::Subscription, InputKind::Image) => "subscription-image",
    };
    let examples = block(
        match surface {
            Surface::Expense => FeatureKind::Expense,
            Surface::Subscription => FeatureKind::Subscription,
        },
        &ctx.today,
    );
    let schema = schema_block(surface, &ctx.today);
    let narration = narration_block(surface, input_kind);

    format!(
        r#"{header}
{narration}

{reply_rule}

{schema}

{precision}

{currency}

{category}

{payment}

{tag}

{dates}
{language}
{examples}"#,
        header = system_header(feature_label),
        narration = narration,
        reply_rule = json_reply_rule(),
        schema = schema,
        precision = precision_rule(),
        currency = currency_rules(&currencies),
        category = category_rules(&categories),
        payment = payment_method_rules(&payments),
        tag = tag_rules(&tags),
        dates = date_rules(&ctx.today),
        language = language,
        examples = examples,
    )
}

/// Schema is per-surface (rows differ) but constant across input kinds —
/// the model fills the same envelope regardless of whether it got text or
/// a picture.
fn schema_block(surface: Surface, today: &str) -> String {
    match surface {
        Surface::Expense => format!(
            r#"Output envelope (all keys camelCase):
{{
  "kind": "receipt" | "statement",
  "transactions": [
    {{
      "name":              string  // merchant / description
      "amount":            number  // positive — grand total for a receipt, per-tx amount for a list
      "currencyCode":      string  // ISO-4217 (see Currency rules)
      "date":              string  // YYYY-MM-DD; if unclear use {today}
      "categoryHint":      string  // optional
      "paymentMethodHint": string  // optional
      "notes":             string  // optional short note
      "tags":              [string] // optional
      "confidence":        number  // 0..1
    }}
  ],
  "metadata": {{
    "merchantName": string         // RECEIPT only: store name
    "totalAmount":  number         // RECEIPT only: grand total
    "lineItems":    [               // RECEIPT only: purchased items (max 20)
      {{ "name": string, "amount": number }}
    ]
  }}
}}

How to choose `kind`:
- "receipt" — ONE purchase event (cashier slip, Apple Pay confirmation,
  bill, single text description). `transactions` has EXACTLY ONE entry;
  populate `metadata.merchantName` + `metadata.totalAmount` and (for
  itemised bills) `metadata.lineItems`.
- "statement" — MULTIPLE independent transactions (bank app history, CSV
  rows, multi-charge text dump). One row per expense, leave `metadata`
  empty (`{{}}`).

Statement-mode tips:
- Skip pending/declined/cancelled and incoming transfers (we only track outflows).
- Negative amounts → take the absolute value.
- If a screenshot has zero transactions, return `{{"kind":"statement","transactions":[]}}`."#
        ),
        Surface::Subscription => String::from(
            r#"Output envelope (all keys camelCase):
{
  "kind": "single" | "list",
  "subscriptions": [
    {
      "name":              string  // service name, e.g. "Telegram Premium"
      "price":             number  // positive periodic price
      "currencyCode":      string  // ISO-4217 (see Currency rules)
      "cycle":             "day" | "week" | "month" | "year"
      "frequency":         integer // billing every N cycles, default 1
      "categoryHint":      string  // optional
      "paymentMethodHint": string  // optional ("Card", "Apple Pay", "PayPal", …)
      "startDate":         string  // optional YYYY-MM-DD
      "nextPayment":       string  // optional YYYY-MM-DD (e.g. "Renews on Jun 12")
      "notes":             string  // optional short note
      "url":               string  // optional
      "tags":              [string] // optional, max 3
      "confidence":        number  // 0..1
    }
  ]
}

How to choose `kind`:
- "single" — exactly ONE service is described (one receipt, one
  confirmation email, one text "Netflix $15/month"). `subscriptions` has
  EXACTLY ONE entry.
- "list" — MULTIPLE distinct subscriptions (Google Play / App Store
  subscriptions screen, Stripe billing portal, PayPal recurring). One row
  per service, max 30 rows.

Cadence:
- "monthly" / "per month" / "/мес" → cycle "month", frequency 1.
- "yearly" / "annual" / "в год" → cycle "year", frequency 1.
- "every N months" → cycle "month", frequency N.
- If only a renewal date is visible without cycle text, guess "month"."#,
        ),
    }
}

/// Narration tells the model *what it's looking at* — the schema stays
/// the same, but a free-form sentence parses very differently from a
/// banking screenshot.
fn narration_block(surface: Surface, input_kind: InputKind) -> &'static str {
    match (surface, input_kind) {
        (Surface::Expense, InputKind::Text) => {
            "You receive FREE-FORM TEXT describing a purchase or several purchases.\n\
             Extract every expense you can confidently identify."
        }
        (Surface::Expense, InputKind::Image) => {
            "You receive ONE PICTURE. It is either:\n\
             * a SINGLE SALES RECEIPT (one purchase, optional line items), or\n\
             * a TRANSACTION LIST screenshot (mobile banking app, history view,\n\
             notification log, in-app statement).\n\
             Decide which kind it is and return the envelope. Skip UI chrome\n\
             (balance widgets, search bars, navigation tabs)."
        }
        (Surface::Subscription, InputKind::Text) => {
            "You receive FREE-FORM TEXT describing one or more recurring services.\n\
             Extract every subscription you can confidently identify."
        }
        (Surface::Subscription, InputKind::Image) => {
            "You receive ONE PICTURE. It is either:\n\
             * a SINGLE subscription confirmation (purchase receipt, welcome\n\
             email, Stripe invoice, app-store charge for one service), or\n\
             * a LIST of active subscriptions (Google Play / App Store / iCloud\n\
             subscriptions screen, browser extension store, Stripe billing\n\
             portal, PayPal recurring payments).\n\
             Decide which kind it is. Skip expired/cancelled rows if visually\n\
             distinct unless the user clearly still tracks them."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ai::dto::CatalogSnapshot;
    use crate::models::{CategoryDoc, CurrencyDoc, PaymentMethodDoc, TagDoc};

    fn ctx_for(locale: Option<&str>) -> ExtractContext {
        let catalogs = CatalogSnapshot {
            categories: Vec::new(),
            currencies: Vec::new(),
            payment_methods: Vec::new(),
            tags: Vec::new(),
        };
        ExtractContext {
            catalogs,
            locale: locale.map(str::to_string),
            today: "2026-05-12".to_string(),
            main_currency_id: "cur-1".to_string(),
        }
    }

    fn ctx_with_catalogs(locale: Option<&str>) -> ExtractContext {
        let catalogs = CatalogSnapshot {
            categories: vec![CategoryDoc {
                id: "c-food".to_string(),
                name: "Food".to_string(),
                icon: String::new(),
                sort_order: 0,
                i18n_key: String::new(),
            }],
            currencies: vec![CurrencyDoc {
                id: "usd".to_string(),
                name: "US Dollar".to_string(),
                symbol: "$".to_string(),
                code: "USD".to_string(),
                rate: 1.0,
                sort_order: 0,
                i18n_key: String::new(),
            }],
            payment_methods: vec![
                PaymentMethodDoc {
                    id: "pm-card".to_string(),
                    name: "Card".to_string(),
                    icon: String::new(),
                    enabled: true,
                    sort_order: 0,
                    i18n_key: String::new(),
                },
                PaymentMethodDoc {
                    id: "pm-disabled".to_string(),
                    name: "Cheque".to_string(),
                    icon: String::new(),
                    enabled: false,
                    sort_order: 1,
                    i18n_key: String::new(),
                },
            ],
            tags: vec![
                TagDoc {
                    id: "t-1".to_string(),
                    name: "Work".to_string(),
                    favorite: true,
                    sort_order: 1,
                    i18n_key: String::new(),
                },
                TagDoc {
                    id: "t-2".to_string(),
                    name: "Coffee".to_string(),
                    favorite: false,
                    sort_order: 2,
                    i18n_key: String::new(),
                },
            ],
        };
        ExtractContext {
            catalogs,
            locale: locale.map(str::to_string),
            today: "2026-05-12".to_string(),
            main_currency_id: "usd".to_string(),
        }
    }

    #[test]
    fn expense_text_prompt_has_receipt_schema() {
        let p = build(&ctx_for(Some("en")), Surface::Expense, InputKind::Text);
        assert!(p.contains("\"kind\": \"receipt\" | \"statement\""));
        assert!(p.contains("transactions"));
        assert!(p.contains("FREE-FORM TEXT"));
    }

    #[test]
    fn expense_image_prompt_describes_picture() {
        let p = build(&ctx_for(Some("ru")), Surface::Expense, InputKind::Image);
        assert!(p.contains("ONE PICTURE"));
        assert!(p.contains("USER_LANGUAGE: Russian"));
    }

    #[test]
    fn subscription_text_prompt_has_single_list_kind() {
        let p = build(&ctx_for(None), Surface::Subscription, InputKind::Text);
        assert!(p.contains("\"kind\": \"single\" | \"list\""));
        assert!(p.contains("subscriptions"));
        assert!(p.contains("frequency"));
    }

    #[test]
    fn subscription_image_prompt_lists_screen_examples() {
        let p = build(&ctx_for(None), Surface::Subscription, InputKind::Image);
        assert!(p.contains("Google Play"));
        assert!(p.contains("App Store"));
    }

    #[test]
    fn prompt_includes_payment_methods_when_present() {
        let p = build(
            &ctx_with_catalogs(Some("en")),
            Surface::Expense,
            InputKind::Text,
        );
        assert!(p.contains("Payment methods:"));
        assert!(p.contains("Card"));
        assert!(!p.contains("Cheque"), "disabled methods must not leak");
    }

    #[test]
    fn prompt_includes_tag_pool_with_favorites_first() {
        let p = build(
            &ctx_with_catalogs(None),
            Surface::Expense,
            InputKind::Image,
        );
        assert!(p.contains("Tags:"));
        let work_pos = p.find("Work").unwrap();
        let coffee_pos = p.find("Coffee").unwrap();
        assert!(
            work_pos < coffee_pos,
            "favourite tag must appear before non-favourite"
        );
    }

    #[test]
    fn prompt_includes_subscription_payment_methods() {
        let p = build(
            &ctx_with_catalogs(None),
            Surface::Subscription,
            InputKind::Text,
        );
        assert!(p.contains("Payment methods:"));
        assert!(p.contains("Card"));
        assert!(p.contains("Tags:"));
    }
}
