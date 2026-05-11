#[derive(Debug, serde::Deserialize)]
struct FixerResponse {
    success: Option<bool>,
    base: Option<String>,
    rates: Option<std::collections::HashMap<String, f64>>,
    error: Option<serde_json::Value>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "fixer",
        name: "Fixer.io",
        requires_key: true,
        free_tier_note: "Free: 100 req/mo, EUR base only",
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
        return Err(crate::errors::AppError::from("fixer api key is required"));
    }
    let mut all = target_codes.to_vec();
    if !all.iter().any(|c| c.eq_ignore_ascii_case(main_code)) {
        all.push(main_code.to_string());
    }
    let symbols = all.join(",");
    // Fixer free tier supports EUR as source only. We fetch EUR-based rates
    // and convert locally to main_code via cross_rate().
    let urls = [
        format!("https://data.fixer.io/api/latest?access_key={}&symbols={}", api_key, symbols),
        format!("http://data.fixer.io/api/latest?access_key={}&symbols={}", api_key, symbols),
    ];

    let mut last_error = String::new();
    for url in urls {
        eprintln!(
            "[subly][rates][fixer] request url={} main_code={} target_codes={:?}",
            url, main_code, target_codes
        );
        let resp = match tauri_plugin_http::reqwest::get(&url).await {
            Ok(v) => v,
            Err(e) => {
                last_error = format!("request failed for {}: {}", url, e);
                eprintln!("[subly][rates][fixer] transport_error={}", last_error);
                continue;
            }
        };
        let status = resp.status();
        let body = resp.text().await.map_err(|e| crate::errors::AppError::Message(e.to_string()))?;
        let body_preview: String = body.chars().take(500).collect();
        eprintln!(
            "[subly][rates][fixer] response status={} body_preview={}",
            status, body_preview
        );
        if !status.is_success() {
            last_error = format!("fixer returned {} for {}: {}", status, url, body);
            continue;
        }
        let parsed: FixerResponse =
            serde_json::from_str(&body).map_err(|e| format!("invalid fixer JSON: {}", e))?;
        if parsed.success == Some(false) {
            last_error = format!(
                "fixer api error: {}",
                parsed
                    .error
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
            continue;
        }
        let rates = parsed.rates.unwrap_or_default();
        let fetch_base = parsed.base.unwrap_or_else(|| "EUR".to_string());
        let mut converted = cross_rate(&rates, &fetch_base, main_code);
        converted.remove(main_code);
        return Ok(converted);
    }

    Err(if last_error.is_empty() {
        crate::errors::AppError::from("fixer provider failed")
    } else {
        crate::errors::AppError::Message(last_error)
    })
}

