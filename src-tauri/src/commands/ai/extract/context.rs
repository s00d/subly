//! [`ExtractContext`] — bundle of everything the LLM extraction pipeline
//! needs once per request.
//!
//! Building one of these takes the state mutex briefly (via
//! [`super::super::catalog_snapshot::snapshot_catalogs`]) and releases it
//! before any `await`. The downstream prompt builder and mapper read from it
//! by reference.

use tauri::State;

use crate::commands::ai::catalog_snapshot::snapshot_catalogs;
use crate::commands::ai::dto::CatalogSnapshot;
use crate::commands::ai::shared::read_main_currency_id;
use crate::errors::AppError;
use crate::AppState;

/// Everything needed to build a prompt and resolve LLM output for one
/// extraction call.
pub struct ExtractContext {
    pub catalogs: CatalogSnapshot,
    /// Used as the fallback when the LLM either omits the currency or supplies
    /// one we don't know. Empty string if the user hasn't set a main currency.
    pub main_currency_id: String,
    /// User's UI language (BCP-47 short tag, e.g. `"ru"`, `"en"`). `None` ⇒
    /// frontend didn't pass one (rare path; we default to English).
    pub locale: Option<String>,
    /// `YYYY-MM-DD` in the user's local timezone. Used as the canonical
    /// "today" anchor for relative dates ("yesterday", "вчера", "heute").
    pub today: String,
}

impl ExtractContext {
    /// Build a context from app state + the per-request `locale` argument.
    /// All blocking reads (state mutex, redb) happen here so subsequent
    /// async work is lock-free.
    pub fn from_state(
        state: &State<'_, AppState>,
        locale: Option<String>,
    ) -> Result<Self, AppError> {
        let catalogs = snapshot_catalogs(state)?;
        let main_currency_id = read_main_currency_id().unwrap_or_default();
        let today = chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        Ok(Self {
            catalogs,
            main_currency_id,
            locale,
            today,
        })
    }

}
