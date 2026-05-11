#[derive(Debug, serde::Deserialize)]
struct FrankfurterResponse {
    rates: std::collections::HashMap<String, f64>,
}

pub fn descriptor() -> super::ProviderDescriptor {
    super::ProviderDescriptor {
        provider_type: "frankfurter",
        name: "Frankfurter",
        requires_key: false,
        free_tier_note: "No API key required",
    }
}

pub async fn fetch_rates(
    main_code: &str,
    _target_codes: &[String],
    _api_key: &str,
) -> Result<std::collections::HashMap<String, f64>, crate::errors::AppError> {
    let urls = [
        format!("https://api.frankfurter.dev/v1/latest?base={}", main_code),
    ];

    let client = tauri_plugin_http::reqwest::Client::new();
    let mut last_error = String::new();

    for url in urls {
        eprintln!(
            "[subly][rates][frankfurter] request url={} main_code={} target_codes={:?}",
            url, main_code, _target_codes
        );
        let ua = format!("Subly/{} (subscription tracker)", env!("CARGO_PKG_VERSION"));
        let resp = match client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", ua)
            .send()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                last_error = format!("request failed for {}: {}", url, e);
                eprintln!("[subly][rates][frankfurter] transport_error={}", last_error);
                continue;
            }
        };

        let status = resp.status();
        let body = match resp.text().await {
            Ok(v) => v,
            Err(e) => {
                last_error = format!("failed reading response body from {}: {}", url, e);
                eprintln!("[subly][rates][frankfurter] body_read_error={}", last_error);
                continue;
            }
        };
        let body_preview: String = body.chars().take(500).collect();
        eprintln!(
            "[subly][rates][frankfurter] response status={} body_preview={}",
            status, body_preview
        );

        if !status.is_success() {
            let snippet: String = body.chars().take(180).collect();
            last_error = format!("{} returned {}: {}", url, status, snippet);
            continue;
        }

        let parsed = match serde_json::from_str::<FrankfurterResponse>(&body) {
            Ok(v) => v,
            Err(e) => {
                let snippet: String = body.chars().take(180).collect();
                last_error = format!("invalid JSON from {}: {} (body: {})", url, e, snippet);
                continue;
            }
        };

        return Ok(parsed.rates);
    }

    Err(if last_error.is_empty() {
        crate::errors::AppError::from("frankfurter provider failed")
    } else {
        crate::errors::AppError::Message(last_error)
    })
}
