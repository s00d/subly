#[derive(Debug, serde::Deserialize)]
struct ExchangeRateResponse {
    result: Option<String>,
    #[serde(rename = "error-type")]
    error_type: Option<String>,
    conversion_rates: Option<std::collections::HashMap<String, f64>>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "exchangerate",
        name: "ExchangeRate-API",
        requires_key: true,
        free_tier_note: "Free: 1500 req/mo, all bases",
    }
}

pub async fn fetch_rates(
    main_code: &str,
    target_codes: &[String],
    api_key: &str,
) -> Result<std::collections::HashMap<String, f64>, crate::errors::AppError> {
    if api_key.trim().is_empty() {
        return Err(crate::errors::AppError::from("exchangerate api key is required"));
    }
    let url = format!(
        "https://v6.exchangerate-api.com/v6/{}/latest/{}",
        api_key, main_code
    );
    eprintln!(
        "[subly][rates][exchangerate] request url={} main_code={} target_codes={:?}",
        url, main_code, target_codes
    );
    let resp = tauri_plugin_http::reqwest::get(&url)
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let body_preview: String = body.chars().take(500).collect();
    eprintln!(
        "[subly][rates][exchangerate] response status={} body_preview={}",
        status, body_preview
    );
    if !status.is_success() {
        return Err(crate::errors::AppError::from(format!(
            "exchangerate returned {}: {}",
            status, body
        )));
    }
    let parsed: ExchangeRateResponse =
        serde_json::from_str(&body).map_err(|e| crate::errors::AppError::from(format!("invalid exchangerate JSON: {}", e)))?;
    if parsed.result.as_deref() != Some("success") {
        return Err(crate::errors::AppError::from(format!(
            "exchangerate api error: {}",
            parsed.error_type.unwrap_or_else(|| "unknown".to_string())
        )));
    }
    let all = parsed.conversion_rates.unwrap_or_default();
    let mut filtered = std::collections::HashMap::new();
    for code in target_codes {
        if let Some(rate) = all.get(code).copied() {
            filtered.insert(code.to_string(), rate);
        }
    }
    Ok(filtered)
}

