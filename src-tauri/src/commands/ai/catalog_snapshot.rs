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

    /// Preferred category from settings if it exists in the snapshot, else `cat-1`
    /// when present, else the first category.
    pub fn resolve_default_category_id(&self, preferred: &str) -> String {
        let preferred = preferred.trim();
        if !preferred.is_empty()
            && self.categories.iter().any(|c| c.id == preferred)
        {
            return preferred.to_string();
        }
        if self
            .categories
            .iter()
            .any(|c| c.id == "cat-1")
        {
            return "cat-1".to_string();
        }
        self.categories
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_default()
    }

    /// Preferred payment method from settings when it exists and is enabled;
    /// otherwise the first enabled method, then any method.
    pub fn resolve_default_payment_method_id(&self, preferred: &str) -> String {
        let preferred = preferred.trim();
        if !preferred.is_empty() {
            if let Some(p) = self.payment_methods.iter().find(|p| p.id == preferred) {
                if p.enabled {
                    return preferred.to_string();
                }
            }
        }
        self.payment_methods
            .iter()
            .find(|p| p.enabled)
            .map(|p| p.id.clone())
            .or_else(|| self.payment_methods.first().map(|p| p.id.clone()))
            .unwrap_or_default()
    }

    /// Keep `id` when it references an existing category; otherwise use `fallback`.
    pub fn ensure_category_id(&self, id: &str, fallback: &str) -> String {
        let id = id.trim();
        if !id.is_empty() && self.categories.iter().any(|c| c.id == id) {
            return id.to_string();
        }
        self.resolve_default_category_id(fallback)
    }

    /// Keep `id` when it references an enabled payment method; otherwise use `fallback`.
    pub fn ensure_payment_method_id(&self, id: &str, fallback: &str) -> String {
        let id = id.trim();
        if !id.is_empty() {
            if let Some(p) = self.payment_methods.iter().find(|p| p.id == id) {
                if p.enabled {
                    return id.to_string();
                }
            }
        }
        self.resolve_default_payment_method_id(fallback)
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

    /// Map LLM tag strings to existing catalog tag names (fuzzy). Unknown
    /// names are kept so the user can create them on save.
    pub fn resolve_tags(&self, raw: Option<Vec<String>>) -> Vec<String> {
        let Some(tags) = raw else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::<String>::new();
        for t in tags {
            let name = self.resolve_tag_name(&t);
            if name.is_empty() {
                continue;
            }
            let key = name.to_lowercase();
            if seen.insert(key) {
                out.push(name);
            }
        }
        out
    }

    fn resolve_tag_name(&self, hint: &str) -> String {
        let h = hint.trim();
        if h.is_empty() {
            return String::new();
        }
        for tag in &self.tags {
            if tag.name.eq_ignore_ascii_case(h) {
                return tag.name.clone();
            }
        }
        let h_lower = h.to_lowercase();
        for tag in &self.tags {
            let name_lc = tag.name.to_lowercase();
            if name_lc.contains(&h_lower) || h_lower.contains(&name_lc) {
                return tag.name.clone();
            }
        }
        h.to_string()
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
            rate: rust_decimal::Decimal::ONE,
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
    fn resolve_tags_maps_to_catalog_names() {
        let mut s = snapshot();
        s.tags = vec![
            tag("t-1", "Work", true, 0),
            tag("t-2", "Coffee", false, 1),
        ];
        let tags = s.resolve_tags(Some(vec![
            "work".to_string(),
            "COFFEE".to_string(),
            "novel".to_string(),
        ]));
        assert_eq!(tags, vec!["Work", "Coffee", "novel"]);
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

    #[test]
    fn resolve_default_category_prefers_valid_settings_id() {
        let mut s = snapshot();
        s.categories.insert(
            0,
            category("cat-1", "No category", "category.none"),
        );
        assert_eq!(s.resolve_default_category_id("c-trans"), "c-trans");
        assert_eq!(s.resolve_default_category_id("cat-missing"), "cat-1");
        assert_eq!(s.resolve_default_category_id(""), "cat-1");
    }

    #[test]
    fn resolve_default_category_without_cat1_uses_first() {
        let s = snapshot();
        assert_eq!(s.resolve_default_category_id("cat-missing"), "c-food");
    }

    #[test]
    fn resolve_default_payment_prefers_enabled_settings_id() {
        let s = snapshot();
        assert_eq!(s.resolve_default_payment_method_id("pm-cash"), "pm-cash");
        assert_eq!(
            s.resolve_default_payment_method_id("pm-disabled"),
            "pm-card"
        );
        assert_eq!(s.resolve_default_payment_method_id(""), "pm-card");
    }

    #[test]
    fn ensure_category_id_rejects_unknown_llm_id() {
        let mut s = snapshot();
        s.categories.insert(
            0,
            category("cat-1", "No category", "category.none"),
        );
        assert_eq!(
            s.ensure_category_id("cat-ghost", "c-trans"),
            "c-trans"
        );
        assert_eq!(s.ensure_category_id("c-food", "c-trans"), "c-food");
    }

    #[test]
    fn ensure_payment_method_id_rejects_disabled_and_unknown() {
        let s = snapshot();
        assert_eq!(
            s.ensure_payment_method_id("pm-disabled", "pm-cash"),
            "pm-cash"
        );
        assert_eq!(
            s.ensure_payment_method_id("pm-ghost", "pm-cash"),
            "pm-cash"
        );
        assert_eq!(s.ensure_payment_method_id("pm-card", "pm-cash"), "pm-card");
    }
}
