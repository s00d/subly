#[derive(Debug, serde::Deserialize)]
struct OpenExchangeRatesResponse {
    error: Option<bool>,
    description: Option<String>,
    base: Option<String>,
    rates: Option<std::collections::HashMap<String, f64>>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "openexchangerates",
        name: "Open Exchange Rates",
        requires_key: true,
        free_tier_note: "Free: 1000 req/mo, USD base only",
    }
}

fn cross_rate(
    rates: &std::collections::HashMap<String, f64>,
    fetch_base: &str,
    target_base: &str,
) -> std::collections::HashMap<String, f64> {
    if fetch_base.eq_ignore_ascii_case(target_base) {
        return rates.clone();
    }
    let Some(base_rate) = rates.get(target_base).copied() else {
        return rates.clone();
    };
    rates
        .iter()
        .map(|(k, v)| (k.clone(), v / base_rate))
        .collect()
}

pub async fn fetch_rates(
    main_code: &str,
    target_codes: &[String],
    api_key: &str,
) -> Result<std::collections::HashMap<String, f64>, crate::errors::AppError> {
    if api_key.trim().is_empty() {
        return Err(crate::errors::AppError::from("openexchangerates api key is required"));
    }
    let mut all = target_codes.to_vec();
    if !all.iter().any(|c| c.eq_ignore_ascii_case(main_code)) {
        all.push(main_code.to_string());
    }
    let symbols = all.join(",");
    // Free plan is USD-base only; omit `base` and convert locally.
    let url = format!(
        "https://openexchangerates.org/api/latest.json?app_id={}&symbols={}",
        api_key, symbols
    );
    eprintln!(
        "[subly][rates][openexchangerates] request url={} main_code={} target_codes={:?}",
        url, main_code, target_codes
    );

    let resp = tauri_plugin_http::reqwest::get(&url)
        .await
        .map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
    let body_preview: String = body.chars().take(500).collect();
    eprintln!(
        "[subly][rates][openexchangerates] response status={} body_preview={}",
        status, body_preview
    );
    if !status.is_success() {
        return Err(crate::errors::AppError::from(format!(
            "openexchangerates returned {}: {}",
            status, body
        )));
    }
    let parsed: OpenExchangeRatesResponse = serde_json::from_str(&body)
        .map_err(|e| crate::errors::AppError::from(format!("invalid openexchangerates JSON: {}", e)))?;
    if parsed.error == Some(true) {
        return Err(crate::errors::AppError::Message(
            parsed
                .description
                .unwrap_or_else(|| "openexchangerates api error".to_string()),
        ));
    }
    let rates = parsed.rates.unwrap_or_default();
    let fetch_base = parsed.base.unwrap_or_else(|| "USD".to_string());
    let mut converted = cross_rate(&rates, &fetch_base, main_code);
    converted.remove(main_code);
    Ok(converted)
}

