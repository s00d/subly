//! Shared infrastructure for the "drop a picture or a file and let the AI
//! turn it into drafts" feature. Both the expense importer
//! ([`super::import`]) and the subscription importer
//! ([`super::import_subscription`]) call into these helpers so the wire
//! format, retries, HEIC handling, and error codes stay in lockstep.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde_json::{json, Value};

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
        let mut out = s[..max].to_string();
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
    let url = format!("{}/chat/completions", active.base_url.trim_end_matches('/'));
    let data_url = format!("data:{};base64,{}", mime, B64.encode(bytes));

    let body = json!({
        "model": active.model,
        "temperature": 0,
        "max_tokens": max_tokens,
        "messages": [
            { "role": "system", "content": system },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": user_instruction },
                    { "type": "image_url", "image_url": { "url": data_url } }
                ]
            }
        ]
    });

    let mut req = reqwest::Client::new().post(&url);
    if !active.api_key.trim().is_empty() {
        req = req.bearer_auth(active.api_key.trim());
    }
    if active.provider_type == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://github.com/subly")
            .header("X-Title", "Subly");
    }
    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::from(format!("{err_prefix}_request:{e}")))?;
    let status = resp.status();
    let raw_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::from(format!(
            "{err_prefix}_http_{}:{}",
            status.as_u16(),
            truncate(&raw_text, 240)
        )));
    }

    let value: Value = serde_json::from_str(&raw_text)
        .map_err(|e| AppError::from(format!("{err_prefix}_response_parse:{e}")))?;
    let content = value
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    if content.is_empty() {
        return Err(AppError::from(format!("{err_prefix}_empty_response")));
    }
    Ok(content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
