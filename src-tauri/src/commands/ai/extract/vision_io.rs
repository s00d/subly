//! Shared infrastructure for the "drop a picture or a file and let the AI
//! turn it into drafts" feature. Both the expense importer
//! ([`super::import`]) and the subscription importer
//! ([`super::import_subscription`]) call into these helpers so the wire
//! format, retries, HEIC handling, and error codes stay in lockstep.

use crate::commands::ai::extract::chat_completions;
use crate::commands::ai::providers::ActiveProvider;
use crate::errors::AppError;

/// MIME types we can hand directly to a vision endpoint. Everything else
/// (HEIC, HEIF) needs to be transcoded first — see
/// [`transcode_heic_to_jpeg`].
const IMAGE_MIME_DIRECT: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/webp",
    "image/gif",
];

/// Returns `true` for any image MIME we can route through the vision
/// branch (after HEIC transcoding if needed).
pub fn is_image_mime(mime: &str) -> bool {
    is_image_direct(mime) || mime == "image/heic" || mime == "image/heif"
}

/// Subset of [`is_image_mime`] that the vision endpoint accepts as-is.
pub fn is_image_direct(mime: &str) -> bool {
    IMAGE_MIME_DIRECT.iter().any(|m| m.eq_ignore_ascii_case(mime))
}

/// Best-effort HEIC → JPEG transcode using the `image` crate. Returns the
/// new bytes plus a `"image/jpeg"` MIME literal so the caller can plug it
/// back into the vision call without re-tagging the buffer.
pub fn transcode_heic_to_jpeg(bytes: &[u8]) -> Result<(Vec<u8>, &'static str), AppError> {
    let dyn_image = image::load_from_memory(bytes)
        .map_err(|_| AppError::from("ai_import_heic_unsupported"))?;
    let rgb = dyn_image.to_rgb8();
    let mut out = std::io::Cursor::new(Vec::new());
    rgb.write_to(&mut out, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::from(format!("ai_import_heic_encode_failed:{e}")))?;
    Ok((out.into_inner(), "image/jpeg"))
}

/// Truncate a string at byte boundary `max`, appending an ellipsis when it
/// was shortened. Used for the bodies we surface in error messages so a
/// 4 MiB HTML 500 page doesn't end up in a toast.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut out = s[..s.floor_char_boundary(max)].to_string();
        out.push('…');
        out
    }
}

/// Direct chat-completions call for vision models. The `system` prompt
/// is feature-specific (receipt vs subscription); the user message is the
/// canonical "classify and extract — reply with ONLY the JSON envelope"
/// instruction plus the base64-data-URL image.
pub async fn call_vision_chat(
    active: &ActiveProvider,
    system: &str,
    user_instruction: &str,
    bytes: &[u8],
    mime: &str,
    max_tokens: u32,
    err_prefix: &'static str,
) -> Result<String, AppError> {
    let json_object = chat_completions::provider_supports_json_object(&active.provider_type);
    chat_completions::call_vision_chat(
        active,
        system,
        user_instruction,
        bytes,
        mime,
        max_tokens,
        err_prefix,
        json_object,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_handles_multibyte_cut() {
        assert_eq!(truncate("абв", 3), "а…");
    }

    #[test]
    fn is_image_mime_covers_direct_and_heic() {
        assert!(is_image_mime("image/png"));
        assert!(is_image_mime("IMAGE/JPEG"));
        assert!(is_image_mime("image/heic"));
        assert!(is_image_mime("image/heif"));
        assert!(!is_image_mime("application/pdf"));
        assert!(!is_image_mime("text/csv"));
    }

    #[test]
    fn truncate_short_stays_unchanged() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_long_appends_ellipsis() {
        let truncated = truncate("0123456789", 4);
        assert_eq!(truncated, "0123…");
    }
}
