use futures::future::join_all;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoAssetDto {
    pub name: String,
    pub path: String,
    pub group: String,
}

fn favicon_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn extract_domain(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };
    let url = tauri_plugin_http::reqwest::Url::parse(&candidate).ok()?;
    let host = url.host_str()?.trim().to_lowercase();
    if host.is_empty() {
        None
    } else {
        Some(host.trim_start_matches("www.").to_string())
    }
}

fn favicon_candidates(domain: &str, size: u32) -> Vec<String> {
    let safe_size = size.clamp(16, 256);
    vec![
        format!("https://{}/favicon.ico", domain),
        format!("https://{}/apple-touch-icon.png", domain),
        format!("https://icons.duckduckgo.com/ip3/{}.ico", domain),
        format!("https://icon.horse/icon/{}", domain),
        format!(
            "https://www.google.com/s2/favicons?domain={}&sz={}",
            domain, safe_size
        ),
    ]
}

async fn url_reachable(url: &str) -> bool {
    let client = tauri_plugin_http::reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(2200))
        .build();
    let Ok(client) = client else {
        return false;
    };
    client
        .get(url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn logo_get_assets() -> Vec<LogoAssetDto> {
    vec![
        LogoAssetDto { name: "Netflix".into(), path: "/assets/netflix.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Spotify".into(), path: "/assets/spotify.svg".into(), group: "service".into() },
        LogoAssetDto { name: "YouTube".into(), path: "/assets/youtube.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Amazon".into(), path: "/assets/amazon.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Disney+".into(), path: "/assets/disney-plus.svg".into(), group: "service".into() },
        LogoAssetDto { name: "HBO".into(), path: "/assets/hbo.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Twitch".into(), path: "/assets/twitch.svg".into(), group: "service".into() },
        LogoAssetDto { name: "GitHub".into(), path: "/assets/github.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Dropbox".into(), path: "/assets/dropbox.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Adobe".into(), path: "/assets/adobe.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Microsoft".into(), path: "/assets/microsoft.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Slack".into(), path: "/assets/slack.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Zoom".into(), path: "/assets/zoom.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Steam".into(), path: "/assets/steam.svg".into(), group: "service".into() },
        LogoAssetDto { name: "PlayStation".into(), path: "/assets/playstation.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Xbox".into(), path: "/assets/xbox.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Nintendo".into(), path: "/assets/nintendo.svg".into(), group: "service".into() },
        LogoAssetDto { name: "iCloud".into(), path: "/assets/icloud.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Notion".into(), path: "/assets/notion.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Figma".into(), path: "/assets/figma.svg".into(), group: "service".into() },
        LogoAssetDto { name: "ChatGPT".into(), path: "/assets/chatgpt.svg".into(), group: "service".into() },
        LogoAssetDto { name: "VPN".into(), path: "/assets/vpn.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Cloud Storage".into(), path: "/assets/cloud-storage.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Music".into(), path: "/assets/music-service.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Gaming".into(), path: "/assets/gaming.svg".into(), group: "service".into() },
        LogoAssetDto { name: "Fitness".into(), path: "/assets/fitness.svg".into(), group: "service".into() },
        LogoAssetDto { name: "PayPal".into(), path: "/assets/paypal.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Visa".into(), path: "/assets/visa.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Visa Alt".into(), path: "/assets/visa-alt.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Mastercard".into(), path: "/assets/mastercard.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Mastercard Alt".into(), path: "/assets/mastercard-alt.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "American Express".into(), path: "/assets/american-express.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Apple Pay".into(), path: "/assets/apple-pay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Google Pay".into(), path: "/assets/google-pay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Samsung Pay".into(), path: "/assets/samsung-pay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Amazon Pay".into(), path: "/assets/amazon-pay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Klarna".into(), path: "/assets/klarna.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "SEPA".into(), path: "/assets/sepa.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Crypto".into(), path: "/assets/crypto.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Maestro".into(), path: "/assets/maestro.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Card Generic".into(), path: "/assets/card-generic.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Card Gold".into(), path: "/assets/card-generic-gold.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Diners Club".into(), path: "/assets/diners.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Discover".into(), path: "/assets/discover.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "JCB".into(), path: "/assets/jcb.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "UnionPay".into(), path: "/assets/unionpay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Alipay".into(), path: "/assets/alipay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "WeChat Pay".into(), path: "/assets/wechat-pay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Bancontact".into(), path: "/assets/bancontact.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "BLIK".into(), path: "/assets/blik.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "EPS".into(), path: "/assets/eps.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Giropay".into(), path: "/assets/giropay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "iDEAL".into(), path: "/assets/ideal.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "MobilePay".into(), path: "/assets/mobilepay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Paysafecard".into(), path: "/assets/paysafecard.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Skrill".into(), path: "/assets/skrill.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Swish".into(), path: "/assets/swish.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "TWINT".into(), path: "/assets/twint.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "V PAY".into(), path: "/assets/vpay.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Invoice".into(), path: "/assets/invoice.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Vipps".into(), path: "/assets/vipps.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Elo".into(), path: "/assets/elo.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Hipercard".into(), path: "/assets/hipercard.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Przelewy24".into(), path: "/assets/przelewy24.svg".into(), group: "payment".into() },
        LogoAssetDto { name: "Dankort".into(), path: "/assets/dankort.svg".into(), group: "payment".into() },
    ]
}

#[tauri::command]
pub async fn logo_resolve_favicon_from_input_url(input: String, size: Option<u32>) -> Result<Option<String>, String> {
    let Some(domain) = extract_domain(&input) else {
        return Ok(None);
    };
    let size = size.unwrap_or(128);
    let cache_key = format!("{}:{}", domain, size);

    if let Ok(cache) = favicon_cache().lock() {
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.clone());
        }
    }

    let candidates = favicon_candidates(&domain, size);
    let checks = candidates
        .into_iter()
        .map(|candidate| async move {
            if url_reachable(&candidate).await {
                Some(candidate)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let resolved = join_all(checks).await.into_iter().flatten().next();

    if let Ok(mut cache) = favicon_cache().lock() {
        cache.insert(cache_key, resolved.clone());
    }
    Ok(resolved)
}
