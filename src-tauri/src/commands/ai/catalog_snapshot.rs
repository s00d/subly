//! Synchronous catalog snapshot helpers.
//!
//! AI commands need a list of categories / currencies / payment methods so
//! the prompt can constrain the model to *existing* IDs. The snapshot is
//! taken under the state mutex, then the lock is released before any `await`
//! so the LLM call doesn't deadlock the rest of the UI.

use crate::commands::ai::dto::CatalogSnapshot;
use crate::errors::AppError;
use crate::models::{CategoryDoc, CurrencyDoc, PaymentMethodDoc, TagDoc};
use crate::state::EntityTable;
use crate::AppState;
use tauri::State;

pub fn snapshot_catalogs(state: &State<'_, AppState>) -> Result<CatalogSnapshot, AppError> {
    let guard = state
        .lock()
        .map_err(|_| AppError::StateLockPoisoned)?;
    let categories: Vec<CategoryDoc> = guard.table_list_typed(EntityTable::Categories)?;
    let currencies: Vec<CurrencyDoc> = guard.table_list_typed(EntityTable::Currencies)?;
    let payment_methods: Vec<PaymentMethodDoc> =
        guard.table_list_typed(EntityTable::PaymentMethods)?;
    let tags: Vec<TagDoc> = guard.table_list_typed(EntityTable::Tags)?;
    Ok(CatalogSnapshot {
        categories,
        currencies,
        payment_methods,
        tags,
    })
}

impl CatalogSnapshot {
    /// Map an LLM-supplied currency code/symbol/name to an internal `currency_id`.
    /// Case-insensitive, falls back to the user's main currency if nothing matches.
    pub fn resolve_currency_id(&self, hint: &str, fallback_id: &str) -> String {
        let h = hint.trim();
        if h.is_empty() {
            return fallback_id.to_string();
        }
        let h_lower = h.to_lowercase();
        // Exact code (e.g. "USD") — most reliable.
        if let Some(c) = self
            .currencies
            .iter()
            .find(|c| !c.code.is_empty() && c.code.eq_ignore_ascii_case(h))
        {
            return c.id.clone();
        }
        // Symbol (₽, $, €, …).
        if let Some(c) = self.currencies.iter().find(|c| c.symbol.trim() == h) {
            return c.id.clone();
        }
        // Name / i18n_key contains.
        if let Some(c) = self.currencies.iter().find(|c| {
            c.name.to_lowercase().contains(&h_lower)
                || c.i18n_key.to_lowercase().contains(&h_lower)
        }) {
            return c.id.clone();
        }
        fallback_id.to_string()
    }

    /// Map a free-form category hint ("entertainment", "Subscriptions",
    /// "Подписки") to an existing `category_id` via case-insensitive contains.
    pub fn resolve_category_id(&self, hint: &str) -> String {
        let h = hint.trim();
        if h.is_empty() {
            return String::new();
        }
        let h_lower = h.to_lowercase();
        for c in &self.categories {
            if c.name.eq_ignore_ascii_case(h) || c.i18n_key.eq_ignore_ascii_case(h) {
                return c.id.clone();
            }
        }
        for c in &self.categories {
            let name_lc = c.name.to_lowercase();
            let key_lc = c.i18n_key.to_lowercase();
            if name_lc.contains(&h_lower)
                || key_lc.contains(&h_lower)
                || h_lower.contains(&name_lc) && !name_lc.is_empty()
            {
                return c.id.clone();
            }
        }
        String::new()
    }

    pub fn resolve_payment_method_id(&self, hint: &str) -> String {
        let h = hint.trim();
        if h.is_empty() {
            return String::new();
        }
        let h_lower = h.to_lowercase();
        for p in &self.payment_methods {
            if !p.enabled {
                continue;
            }
            if p.name.eq_ignore_ascii_case(h) || p.i18n_key.eq_ignore_ascii_case(h) {
                return p.id.clone();
            }
        }
        for p in &self.payment_methods {
            if !p.enabled {
                continue;
            }
            if p.name.to_lowercase().contains(&h_lower)
                || p.i18n_key.to_lowercase().contains(&h_lower)
            {
                return p.id.clone();
            }
        }
        String::new()
    }

    /// Format category list for inclusion in a system prompt.
    pub fn render_category_names(&self) -> String {
        let mut names: Vec<&str> = self
            .categories
            .iter()
            .map(|c| c.name.as_str())
            .filter(|n| !n.is_empty())
            .collect();
        names.sort_unstable();
        names.dedup();
        names.join(", ")
    }

    /// Format currency codes for inclusion in a system prompt.
    pub fn render_currency_codes(&self) -> String {
        let mut codes: Vec<&str> = self
            .currencies
            .iter()
            .map(|c| c.code.as_str())
            .filter(|c| !c.is_empty())
            .collect();
        codes.sort_unstable();
        codes.dedup();
        codes.join(", ")
    }

    /// Format enabled payment methods for inclusion in a system prompt.
    /// Disabled methods are dropped because the model shouldn't propose them.
    pub fn render_payment_method_names(&self) -> String {
        let mut names: Vec<&str> = self
            .payment_methods
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.name.as_str())
            .filter(|n| !n.is_empty())
            .collect();
        names.sort_unstable();
        names.dedup();
        names.join(", ")
    }

    /// Format tag list for inclusion in a system prompt.
    ///
    /// Strategy: favourites first (alphabetised inside the favourite group),
    /// then everyone else by `sort_order`, capped at `TAG_PROMPT_CAP` so the
    /// system prompt stays light on tokens for users with hundreds of tags.
    pub fn render_tag_names(&self) -> String {
        const TAG_PROMPT_CAP: usize = 30;
        let mut tags: Vec<&crate::models::TagDoc> = self
            .tags
            .iter()
            .filter(|t| !t.name.trim().is_empty())
            .collect();
        // (-favourite, sort_order, name) — favourites first, stable inside each group.
        tags.sort_by(|a, b| {
            (!a.favorite)
                .cmp(&!b.favorite)
                .then_with(|| a.sort_order.cmp(&b.sort_order))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        let mut seen = std::collections::HashSet::<String>::new();
        let mut names: Vec<&str> = Vec::with_capacity(tags.len().min(TAG_PROMPT_CAP));
        for tag in tags.iter() {
            let key = tag.name.to_lowercase();
            if seen.insert(key) {
                names.push(tag.name.as_str());
                if names.len() >= TAG_PROMPT_CAP {
                    break;
                }
            }
        }
        names.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CategoryDoc, CurrencyDoc, PaymentMethodDoc};

    fn currency(id: &str, code: &str, symbol: &str, name: &str) -> CurrencyDoc {
        CurrencyDoc {
            id: id.to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            code: code.to_string(),
            rate: 1.0,
            sort_order: 0,
            i18n_key: String::new(),
        }
    }

    fn category(id: &str, name: &str, i18n: &str) -> CategoryDoc {
        CategoryDoc {
            id: id.to_string(),
            name: name.to_string(),
            icon: String::new(),
            sort_order: 0,
            i18n_key: i18n.to_string(),
        }
    }

    fn payment(id: &str, name: &str, enabled: bool) -> PaymentMethodDoc {
        PaymentMethodDoc {
            id: id.to_string(),
            name: name.to_string(),
            icon: String::new(),
            enabled,
            sort_order: 0,
            i18n_key: String::new(),
        }
    }

    fn snapshot() -> CatalogSnapshot {
        CatalogSnapshot {
            categories: vec![
                category("c-food", "Food", "category.food"),
                category("c-trans", "Transport", "category.transport"),
            ],
            currencies: vec![
                currency("usd", "USD", "$", "US Dollar"),
                currency("eur", "EUR", "€", "Euro"),
                currency("rub", "RUB", "₽", "Russian Ruble"),
            ],
            payment_methods: vec![
                payment("pm-card", "Card", true),
                payment("pm-cash", "Cash", true),
                payment("pm-disabled", "Cheque", false),
            ],
            tags: vec![],
        }
    }

    fn tag(id: &str, name: &str, favorite: bool, sort_order: i64) -> crate::models::TagDoc {
        crate::models::TagDoc {
            id: id.to_string(),
            name: name.to_string(),
            favorite,
            sort_order,
            i18n_key: String::new(),
        }
    }

    #[test]
    fn resolve_currency_exact_code_match() {
        let s = snapshot();
        assert_eq!(s.resolve_currency_id("USD", ""), "usd");
        assert_eq!(s.resolve_currency_id("usd", ""), "usd"); // case-insensitive
    }

    #[test]
    fn resolve_currency_symbol_match() {
        let s = snapshot();
        assert_eq!(s.resolve_currency_id("₽", ""), "rub");
        assert_eq!(s.resolve_currency_id("$", ""), "usd");
    }

    #[test]
    fn resolve_currency_name_substring_match() {
        let s = snapshot();
        assert_eq!(s.resolve_currency_id("dollar", ""), "usd");
        assert_eq!(s.resolve_currency_id("Euro", ""), "eur");
    }

    #[test]
    fn resolve_currency_falls_back_when_unknown() {
        let s = snapshot();
        assert_eq!(s.resolve_currency_id("XYZ", "usd"), "usd");
        assert_eq!(s.resolve_currency_id("", "usd"), "usd");
        assert_eq!(s.resolve_currency_id("XYZ", ""), "");
    }

    #[test]
    fn resolve_category_exact_then_substring() {
        let s = snapshot();
        assert_eq!(s.resolve_category_id("Food"), "c-food");
        assert_eq!(s.resolve_category_id("food"), "c-food"); // case-insensitive substring
        assert_eq!(s.resolve_category_id("Groceries (Food)"), "c-food"); // input contains name
        assert_eq!(s.resolve_category_id("category.transport"), "c-trans"); // i18n_key match
        assert_eq!(s.resolve_category_id(""), "");
        assert_eq!(s.resolve_category_id("Entertainment"), "");
    }

    #[test]
    fn resolve_payment_method_skips_disabled() {
        let s = snapshot();
        assert_eq!(s.resolve_payment_method_id("Card"), "pm-card");
        assert_eq!(s.resolve_payment_method_id("cash"), "pm-cash"); // case-insensitive
        // disabled method must not be returned even on exact match
        assert_eq!(s.resolve_payment_method_id("Cheque"), "");
        assert_eq!(s.resolve_payment_method_id(""), "");
    }

    #[test]
    fn render_currency_codes_sorted_and_deduped() {
        let s = snapshot();
        // Sorted alphabetically
        assert_eq!(s.render_currency_codes(), "EUR, RUB, USD");
    }

    #[test]
    fn render_category_names_sorted() {
        let s = snapshot();
        assert_eq!(s.render_category_names(), "Food, Transport");
    }

    #[test]
    fn render_payment_methods_drops_disabled() {
        let s = snapshot();
        assert_eq!(s.render_payment_method_names(), "Card, Cash");
    }

    #[test]
    fn render_tag_names_favorites_first_then_sort_order() {
        let mut s = snapshot();
        s.tags = vec![
            tag("t-a", "Coffee", false, 30),
            tag("t-b", "Work", true, 5),
            tag("t-c", "Snack", false, 10),
            tag("t-d", "Travel", true, 20),
        ];
        assert_eq!(s.render_tag_names(), "Work, Travel, Snack, Coffee");
    }

    #[test]
    fn render_tag_names_caps_at_thirty_and_dedupes() {
        let mut s = snapshot();
        s.tags = (0..40)
            .map(|i| tag(&format!("t-{i}"), &format!("tag-{i:02}"), false, i))
            .collect();
        // Add a duplicate (case-insensitive) — must be dropped.
        s.tags.push(tag("dup", "Tag-00", false, 100));
        let rendered = s.render_tag_names();
        let names: Vec<&str> = rendered.split(", ").collect();
        assert_eq!(names.len(), 30);
        assert_eq!(names.first(), Some(&"tag-00"));
        assert_eq!(names.last(), Some(&"tag-29"));
    }

    #[test]
    fn render_tag_names_skips_blank_names() {
        let mut s = snapshot();
        s.tags = vec![
            tag("t-1", "  ", false, 1),
            tag("t-2", "Important", true, 2),
            tag("t-3", "", false, 3),
        ];
        assert_eq!(s.render_tag_names(), "Important");
    }
}
