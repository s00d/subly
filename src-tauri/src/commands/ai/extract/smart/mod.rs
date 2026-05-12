//! `ai_smart_input` — the only AI-extraction command exposed to the
//! frontend. Replaces the previous quartet (text-expense, text-subscription,
//! file-expense, file-subscription) with one entry point that takes a
//! `surface` ("expense" | "subscription") plus *either* a free-form text
//! *or* file bytes and figures out the rest.
//!
//! Dispatch (4-way matrix):
//! ```text
//!                 ┌───────────── surface ─────────────┐
//!                 │  Expense           Subscription   │
//! ┌───────────────┼──────────────────────────────────┤
//! │  text         │ smart-prompt + run_text          │
//! │  image MIME   │ smart-prompt + call_vision_chat  │
//! │  tabular MIME │ heuristics +        smart-prompt │
//! │               │ statement-chunk     on full text │
//! └───────────────┴──────────────────────────────────┘
//! ```
//!
//! The returned [`AiSmartResultDto`] is a tagged enum (surface = "expense"
//! | "subscription"), so the frontend feeds it to one of two preview
//! lists without ever switching on the file kind.
//!
//! Sub-modules:
//! * [`expense`] — full pipeline for the expense surface (incl. the
//!   heuristics-fallback tabular branch).
//! * [`subscription`] — full pipeline for the subscription surface.

mod expense;
mod subscription;

use tauri::ipc::Channel;
use tauri::State;

use crate::commands::ai::dto::{AiSmartResultDto, StatementImportProgress};
use crate::commands::ai::extract::context::ExtractContext;
use crate::commands::ai::extract::vision_io::{
    is_image_direct, is_image_mime, transcode_heic_to_jpeg,
};
use crate::commands::ai::prompts::smart::Surface;
use crate::commands::ai::shared::{require_feature_enabled, AiFeature};
use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    Text,
    Image,
    Tabular,
}

#[tauri::command]
pub async fn ai_smart_input(
    state: State<'_, AppState>,
    surface: String,
    text: String,
    bytes: Vec<u8>,
    mime: String,
    locale: Option<String>,
    on_progress: Channel<StatementImportProgress>,
) -> Result<AiSmartResultDto, AppError> {
    let surface = parse_surface(&surface)?;
    let trimmed_text = text.trim().to_string();
    let has_text = !trimmed_text.is_empty();
    let has_bytes = !bytes.is_empty();
    if !has_text && !has_bytes {
        return Err(AppError::from("ai_input_empty"));
    }

    let mime_lc = mime.to_lowercase();
    let route = if has_bytes {
        if is_image_mime(&mime_lc) {
            Route::Image
        } else {
            Route::Tabular
        }
    } else {
        Route::Text
    };

    // Each (surface, route) hits a specific feature gate so a user that
    // disabled only one capability (e.g. "receipt import") doesn't lose
    // every adjacent flow.
    let feature = match (surface, route) {
        (Surface::Expense, Route::Text) => AiFeature::ExpenseInput,
        (Surface::Expense, Route::Image) => AiFeature::ReceiptImport,
        (Surface::Expense, Route::Tabular) => AiFeature::StatementImport,
        (Surface::Subscription, _) => AiFeature::SubscriptionInput,
    };
    require_feature_enabled(feature)?;

    let ctx = ExtractContext::from_state(&state, locale)?;

    match (surface, route) {
        (Surface::Expense, Route::Text) => {
            expense::run_text(&trimmed_text, &ctx, &on_progress).await
        }
        (Surface::Expense, Route::Image) => {
            let (img_bytes, img_mime) = prepare_image(bytes, &mime_lc)?;
            expense::run_image(&img_bytes, &img_mime, &ctx, &on_progress).await
        }
        (Surface::Expense, Route::Tabular) => {
            expense::run_tabular(&bytes, &mime, &ctx, &on_progress).await
        }
        (Surface::Subscription, Route::Text) => {
            subscription::run_text(&trimmed_text, &ctx, &on_progress).await
        }
        (Surface::Subscription, Route::Image) => {
            let (img_bytes, img_mime) = prepare_image(bytes, &mime_lc)?;
            subscription::run_image(&img_bytes, &img_mime, &ctx, &on_progress).await
        }
        (Surface::Subscription, Route::Tabular) => {
            subscription::run_tabular(&bytes, &mime, &ctx, &on_progress).await
        }
    }
}

fn parse_surface(raw: &str) -> Result<Surface, AppError> {
    match raw.trim().to_lowercase().as_str() {
        "expense" | "expenses" => Ok(Surface::Expense),
        "subscription" | "subscriptions" => Ok(Surface::Subscription),
        other => Err(AppError::from(format!("ai_smart_unknown_surface:{other}"))),
    }
}

/// Normalise an image buffer into `(bytes, mime)` the vision endpoint can
/// accept directly. HEIC/HEIF gets transcoded to JPEG; anything else is
/// passed through untouched.
fn prepare_image(bytes: Vec<u8>, mime_lc: &str) -> Result<(Vec<u8>, String), AppError> {
    if is_image_direct(mime_lc) {
        return Ok((bytes, mime_lc.to_string()));
    }
    if mime_lc == "image/heic" || mime_lc == "image/heif" {
        let (jpeg, jpeg_mime) = transcode_heic_to_jpeg(&bytes)?;
        return Ok((jpeg, jpeg_mime.to_string()));
    }
    Err(AppError::from(format!("ai_import_unsupported_mime:{mime_lc}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_surface_accepts_known_values() {
        assert_eq!(parse_surface("expense").unwrap(), Surface::Expense);
        assert_eq!(parse_surface("EXPENSES").unwrap(), Surface::Expense);
        assert_eq!(parse_surface("subscription").unwrap(), Surface::Subscription);
        assert_eq!(parse_surface("Subscriptions").unwrap(), Surface::Subscription);
        assert!(parse_surface("nonsense").is_err());
    }
}
