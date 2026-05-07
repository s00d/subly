#[derive(Debug, serde::Deserialize)]
struct ApiLayerResponse {
    success: Option<bool>,
    base: Option<String>,
    rates: Option<std::collections::HashMap<String, f64>>,
    error: Option<serde_json::Value>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "apilayer",
        name: "APILayer (Fixer)",
        requires_key: true,
        free_tier_note: "Free: 100 req/mo",
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
) -> Result<std::collections::HashMap<String, f64>, String> {
    if api_key.trim().is_empty() {
        return Err("apilayer api key is required".to_string());
    }
    let mut all = target_codes.to_vec();
    if !all.iter().any(|c| c.eq_ignore_ascii_case(main_code)) {
        all.push(main_code.to_string());
    }
    let symbols = all.join(",");
    // On lower/free plans, custom `base` may be restricted (EUR-only).
    // Fetch provider-default base and convert locally via cross_rate().
    let url = format!(
        "https://api.apilayer.com/fixer/latest?symbols={}",
        symbols
    );
    eprintln!(
        "[subly][rates][apilayer] request url={} main_code={} target_codes={:?}",
        url, main_code, target_codes
    );
    let client = tauri_plugin_http::reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("apikey", api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    let body_preview: String = body.chars().take(500).collect();
    eprintln!(
        "[subly][rates][apilayer] response status={} body_preview={}",
        status, body_preview
    );
    if !status.is_success() {
        return Err(format!("apilayer returned {}: {}", status, body));
    }
    let parsed: ApiLayerResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid apilayer JSON: {}", e))?;
    if parsed.success == Some(false) {
        return Err(format!(
            "apilayer api error: {}",
            parsed
                .error
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }
    let rates = parsed.rates.unwrap_or_default();
    let fetch_base = parsed.base.unwrap_or_else(|| "EUR".to_string());
    let mut converted = cross_rate(&rates, &fetch_base, main_code);
    converted.remove(main_code);
    Ok(converted)
}

