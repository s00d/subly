#[derive(Debug, serde::Deserialize)]
struct CurrencyApiValue {
    value: f64,
}

#[derive(Debug, serde::Deserialize)]
struct CurrencyApiResponse {
    data: Option<std::collections::HashMap<String, CurrencyApiValue>>,
    errors: Option<serde_json::Value>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "currencyapi",
        name: "CurrencyAPI",
        requires_key: true,
        free_tier_note: "Free: 300 req/mo, all bases",
    }
}

pub async fn fetch_rates(
    main_code: &str,
    target_codes: &[String],
    api_key: &str,
) -> Result<std::collections::HashMap<String, f64>, String> {
    if api_key.trim().is_empty() {
        return Err("currencyapi api key is required".to_string());
    }
    let currencies = target_codes.join(",");
    let url = format!(
        "https://api.currencyapi.com/v3/latest?base_currency={}&currencies={}",
        main_code, currencies
    );
    eprintln!(
        "[subly][rates][currencyapi] request url={} main_code={} target_codes={:?}",
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
        "[subly][rates][currencyapi] response status={} body_preview={}",
        status, body_preview
    );
    if !status.is_success() {
        return Err(format!("currencyapi returned {}: {}", status, body));
    }
    let parsed: CurrencyApiResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid currencyapi JSON: {}", e))?;
    if let Some(err) = parsed.errors {
        return Err(format!("currencyapi api error: {}", err));
    }
    let mut out = std::collections::HashMap::new();
    for (code, payload) in parsed.data.unwrap_or_default() {
        out.insert(code, payload.value);
    }
    Ok(out)
}

