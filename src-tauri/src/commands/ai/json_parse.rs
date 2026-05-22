//! Robust JSON extraction from LLM replies.
//!
//! LLMs frequently wrap JSON in markdown fences or add surrounding prose
//! despite explicit instructions. We strip those wrappers before handing the
//! text to `serde_json`.
//!
//! When multiple JSON values appear in one reply (e.g. an example in the
//! preamble and the real payload at the end), we extract every balanced
//! `{…}` / `[…]` chunk (respecting string literals) and deserialize
//! **last successful** match first — the final object is usually the answer.

use serde::de::DeserializeOwned;

use crate::errors::AppError;

/// Parse JSON out of a free-form LLM response.
pub fn parse_llm_json<T: DeserializeOwned>(raw: &str) -> Result<T, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::from("ai_response_empty"));
    }

    // Fast path: the model returned bare JSON.
    if let Ok(value) = serde_json::from_str::<T>(trimmed) {
        return Ok(value);
    }

    // Try to extract a fenced code block (```json … ``` or ``` … ```).
    if let Some(stripped) = strip_code_fence(trimmed) {
        if let Ok(value) = serde_json::from_str::<T>(stripped) {
            return Ok(value);
        }
    }

    // Try every balanced JSON value in the text; prefer the last one that parses.
    let candidates = collect_json_candidates(trimmed);
    let mut last_err: Option<String> = None;
    for chunk in candidates.iter().rev() {
        match serde_json::from_str::<T>(chunk) {
            Ok(value) => return Ok(value),
            Err(e) => last_err = Some(truncate_message(&e.to_string())),
        }
    }

    // First balanced chunk only (forward order) for odd edge cases.
    if let Some(start) = trimmed.find(|c: char| c == '{' || c == '[') {
        if let Some(json_chunk) = extract_balanced_json(trimmed, start) {
            return serde_json::from_str::<T>(&json_chunk).map_err(|e| {
                AppError::from(format!(
                    "ai_response_invalid_json:{}",
                    truncate_message(&e.to_string())
                ))
            });
        }
    }

    Err(AppError::from(format!(
        "ai_response_invalid_json:{}",
        last_err.unwrap_or_else(|| truncate_message(trimmed))
    )))
}

fn strip_code_fence(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let after_open = trimmed.trim_start_matches('`').trim_start();
    let after_tag = match after_open.find('\n') {
        Some(idx) => &after_open[idx + 1..],
        None => after_open,
    };
    let end = after_tag.rfind("```")?;
    Some(after_tag[..end].trim())
}

/// All top-level JSON object/array spans found in `raw`, in left-to-right order.
fn collect_json_candidates(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, ch) in raw.char_indices() {
        if ch == '{' || ch == '[' {
            if let Some(chunk) = extract_balanced_json(raw, idx) {
                out.push(chunk);
            }
        }
    }
    out
}

/// Extract one balanced JSON value starting at byte index `start` (`{` or `[`).
fn extract_balanced_json(raw: &str, start: usize) -> Option<String> {
    let first = *raw.as_bytes().get(start)?;
    if first != b'{' && first != b'[' {
        return None;
    }

    let mut depth_obj: i32 = 0;
    let mut depth_arr: i32 = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in raw[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => in_string = true,
            '{' => depth_obj += 1,
            '}' => {
                depth_obj -= 1;
                if depth_obj == 0 && depth_arr == 0 {
                    let end = start + i + c.len_utf8();
                    return Some(raw[start..end].to_string());
                }
            }
            '[' => depth_arr += 1,
            ']' => {
                depth_arr -= 1;
                if depth_obj == 0 && depth_arr == 0 {
                    let end = start + i + c.len_utf8();
                    return Some(raw[start..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn truncate_message(s: &str) -> String {
    let mut out: String = s.chars().take(200).collect();
    if s.chars().count() > 200 {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        name: String,
        amount: f64,
    }

    #[test]
    fn parses_bare_json() {
        let result: Sample = parse_llm_json(r#"{"name":"foo","amount":12.5}"#).unwrap();
        assert_eq!(
            result,
            Sample {
                name: "foo".to_string(),
                amount: 12.5
            }
        );
    }

    #[test]
    fn strips_markdown_fence_with_language_tag() {
        let raw = "```json\n{\"name\":\"bar\",\"amount\":1}\n```";
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "bar");
        assert_eq!(result.amount, 1.0);
    }

    #[test]
    fn strips_plain_markdown_fence() {
        let raw = "```\n{\"name\":\"baz\",\"amount\":2}\n```";
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "baz");
    }

    #[test]
    fn extracts_json_surrounded_by_prose() {
        let raw = "Sure, here's the result:\n{\"name\":\"qux\",\"amount\":3.14}\nLet me know.";
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "qux");
        assert_eq!(result.amount, 3.14);
    }

    #[test]
    fn handles_braces_inside_string_literal() {
        let raw = r#"Reply: {"name":"with {braces}","amount":1.0}"#;
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "with {braces}");
    }

    #[test]
    fn handles_escaped_quotes() {
        let raw = r#"{"name":"she said \"hi\"","amount":5.0}"#;
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "she said \"hi\"");
    }

    #[test]
    fn prefers_last_valid_object_when_prose_contains_example() {
        let raw = r#"Example: {"name":"wrong","amount":0}
Here is the extracted row:
{"name":"right","amount":42}"#;
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "right");
        assert_eq!(result.amount, 42.0);
    }

    #[test]
    fn parses_nested_array_inside_object() {
        let raw = r#"{"name":"nested","amount":1,"tags":["a","b"]}"#;
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "nested");
    }

    #[test]
    fn returns_descriptive_error_on_empty_input() {
        let err = parse_llm_json::<Sample>("").unwrap_err();
        assert!(err.to_string().contains("ai_response_empty"));
    }

    #[test]
    fn returns_descriptive_error_on_unparseable_input() {
        let err = parse_llm_json::<Sample>("totally not json").unwrap_err();
        assert!(err.to_string().contains("ai_response_invalid_json"));
    }

    #[test]
    fn parses_array_outer_type() {
        let result: Vec<u32> = parse_llm_json("garbage before [1, 2, 3] garbage after").unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn notes_field_with_embedded_json_does_not_break_outer_object() {
        let raw = r#"{"name":"ok","amount":9,"notes":"user pasted {\"foo\": 1} here"}"#;
        let result: Sample = parse_llm_json(raw).unwrap();
        assert_eq!(result.name, "ok");
        assert_eq!(result.amount, 9.0);
    }
}
