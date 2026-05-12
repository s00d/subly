//! Few-shot exemplars per feature.
//!
//! Each block is appended after the schema + rules so the model has a
//! concrete I/O pattern to imitate. We deliberately include a Russian
//! example because LLMs frequently switch back to English for `name` if all
//! examples are English.
//!
//! Examples are kept short — every extra token costs latency on local
//! Ollama installs.

#[derive(Debug, Clone, Copy)]
pub enum FeatureKind {
    Subscription,
    Expense,
    Receipt,
    Statement,
}

/// Return a self-contained `Examples:\n…` block.
///
/// `today` (YYYY-MM-DD) is used to materialise the `<today>` / `<yesterday>`
/// placeholders in date-aware examples so the model sees concrete dates.
pub fn block(feature: FeatureKind, today: &str) -> String {
    let yesterday = previous_day(today).unwrap_or_else(|| today.to_string());
    match feature {
        FeatureKind::Subscription => SUBSCRIPTION.to_string(),
        FeatureKind::Expense => EXPENSE
            .replace("<today>", today)
            .replace("<yesterday>", &yesterday),
        FeatureKind::Receipt => RECEIPT.to_string(),
        FeatureKind::Statement => STATEMENT.to_string(),
    }
}

/// Subtract one day from a `YYYY-MM-DD` anchor. Returns `None` for unparseable
/// inputs (caller falls back to `today`).
fn previous_day(today: &str) -> Option<String> {
    let date = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d").ok()?;
    let prev = date.pred_opt()?;
    Some(prev.format("%Y-%m-%d").to_string())
}

const SUBSCRIPTION: &str = r#"
Examples:

Input: "Telegram Premium 300 rub monthly, card"
JSON: {"name":"Telegram Premium","price":300,"currencyCode":"RUB","cycle":"month","frequency":1,"paymentMethodHint":"Card","confidence":0.95}

Input: "Подписка на Spotify 169р в месяц"
JSON: {"name":"Spotify","price":169,"currencyCode":"RUB","cycle":"month","frequency":1,"confidence":0.92}

Input: "Notion Plus $10/month, billed yearly $96"
JSON: {"name":"Notion Plus","price":96,"currencyCode":"USD","cycle":"year","frequency":1,"notes":"Billed yearly ($10/mo)","confidence":0.85}
"#;

const EXPENSE: &str = r#"
Examples:

Input: "Coffee at Starbucks 5.40 EUR yesterday, card"
JSON: {"name":"Starbucks coffee","amount":5.4,"currencyCode":"EUR","date":"<yesterday>","categoryHint":"Food","paymentMethodHint":"Card","confidence":0.9}

Input: "Такси 450р вчера, наличными"
JSON: {"name":"Такси","amount":450,"currencyCode":"RUB","date":"<yesterday>","categoryHint":"Transport","paymentMethodHint":"Cash","confidence":0.9}

Input: "Domain renewal hover.com 14.99 today"
JSON: {"name":"hover.com domain renewal","amount":14.99,"currencyCode":"USD","date":"<today>","url":"https://hover.com","confidence":0.85}
"#;

const RECEIPT: &str = r#"
Examples:

Receipt (image): grocery receipt, total $42.50
JSON: {"name":"Whole Foods","amount":42.5,"currencyCode":"USD","categoryHint":"Groceries","paymentMethodHint":"Card","confidence":0.9,"lineItems":[{"name":"Bananas","amount":3.5},{"name":"Bread","amount":4.2}]}

Receipt (PDF text): "Магнит\n01.05.2026\nИтого: 1 234,56 ₽"
JSON: {"name":"Магнит","amount":1234.56,"currencyCode":"RUB","date":"2026-05-01","categoryHint":"Groceries","confidence":0.9}
"#;

const STATEMENT: &str = r#"
Examples:

Lines:
2026-04-30 STARBUCKS 5.40 USD
2026-04-29 UBER 12.30 USD CARD ****1234

JSON: {"transactions":[
  {"name":"Starbucks","amount":5.4,"currencyCode":"USD","date":"2026-04-30","categoryHint":"Food","confidence":0.9},
  {"name":"Uber","amount":12.3,"currencyCode":"USD","date":"2026-04-29","categoryHint":"Transport","paymentMethodHint":"Card","confidence":0.9}
]}

Lines:
01.05.26 Магнит -1234.56 RUB
01.05.26 OZON -3199 RUB

JSON: {"transactions":[
  {"name":"Магнит","amount":1234.56,"currencyCode":"RUB","date":"2026-05-01","categoryHint":"Groceries","confidence":0.9},
  {"name":"OZON","amount":3199,"currencyCode":"RUB","date":"2026-05-01","confidence":0.85}
]}
"#;
