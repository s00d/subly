//! Robust JSON extraction from LLM replies.
//!
//! LLMs frequently wrap JSON in markdown fences or add surrounding prose
//! despite explicit instructions. We strip those wrappers before handing the
//! text to `serde_json`.

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

    // Final fallback: take the substring from the first `{`/`[` to the
    // matching closing bracket, trimming prose around it.
    if let Some(json_chunk) = extract_outer_json(trimmed) {
        return serde_json::from_str::<T>(&json_chunk).map_err(|e| {
            AppError::from(format!(
                "ai_response_invalid_json:{}",
                truncate_message(&e.to_string())
            ))
        });
    }

    Err(AppError::from(format!(
        "ai_response_invalid_json:{}",
        truncate_message(trimmed)
    )))
}

fn strip_code_fence(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let after_open = trimmed.trim_start_matches('`').trim_start();
    // Skip optional language tag (e.g. `json` / `JSON`).
    let after_tag = match after_open.find('\n') {
        Some(idx) => &after_open[idx + 1..],
        None => after_open,
    };
    let end = after_tag.rfind("```")?;
    Some(after_tag[..end].trim())
}

fn extract_outer_json(raw: &str) -> Option<String> {
    let start = raw.find(|c: char| c == '{' || c == '[')?;
    let open = raw.as_bytes()[start] as char;
    let close = if open == '{' { '}' } else { ']' };
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut prev_escape = false;
    for (i, c) in raw[start..].char_indices() {
        if in_string {
            if prev_escape {
                prev_escape = false;
            } else if c == '\\' {
                prev_escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            ch if ch == open => depth += 1,
            ch if ch == close => {
                depth -= 1;
                if depth == 0 {
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
        // The outer-extractor must skip the `{` inside the `notes` string
        // and find the actual closing brace.
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
}
