use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::RngExt;
use sha2::{Digest, Sha256};

use super::{
    load_oauth_tokens, now_ts, save_oauth_tokens, token_key, OAuthTokens, SyncConfig, SyncProviderType,
    OAUTH_REDIRECT_URI,
};
use super::config::{dropbox_app_secret, gdrive_oauth_client_id};
use super::state::sync_runtime;

fn enc_query(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

fn random_pkce_verifier() -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge_s256(verifier: &str) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Builds the Google authorization URL and stores the PKCE verifier in [`super::state::SyncRuntime`].
pub(crate) fn google_authorize_url() -> Result<String, String> {
    let verifier = random_pkce_verifier();
    let challenge = pkce_challenge_s256(&verifier);
    {
        let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
        rt.oauth_pkce_verifier = Some(verifier);
    }
    let client_id = gdrive_oauth_client_id();
    let scope = "https://www.googleapis.com/auth/drive.appdata";
    Ok(format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state=gdrive&access_type=offline&prompt=consent&code_challenge={}&code_challenge_method=S256",
        enc_query(client_id),
        enc_query(OAUTH_REDIRECT_URI),
        enc_query(scope),
        enc_query(&challenge),
    ))
}

#[derive(Debug, serde::Deserialize)]
struct OAuthTokenResponse {
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default = "default_expires_in")]
    expires_in: i64,
}

fn default_expires_in() -> i64 {
    3600
}

pub async fn exchange_oauth_code(
    provider: &SyncProviderType,
    cfg: &SyncConfig,
    code: &str,
) -> Result<OAuthTokens, String> {
    let client = tauri_plugin_http::reqwest::Client::new();
    let redirect_uri = OAUTH_REDIRECT_URI;
    let (url, body) = match provider {
        SyncProviderType::Gdrive => {
            let verifier = {
                let mut rt = sync_runtime().lock().map_err(|_| "sync runtime lock poisoned".to_string())?;
                rt.oauth_pkce_verifier.take()
            }
            .ok_or_else(|| {
                "google oauth: PKCE verifier missing — open Connect again and complete sign-in in one flow"
                    .to_string()
            })?;
            (
                "https://oauth2.googleapis.com/token",
                vec![
                    ("code", code.to_string()),
                    ("client_id", gdrive_oauth_client_id().to_string()),
                    ("code_verifier", verifier),
                    ("redirect_uri", redirect_uri.to_string()),
                    ("grant_type", "authorization_code".to_string()),
                ],
            )
        }
        SyncProviderType::Dropbox => (
            "https://api.dropbox.com/oauth2/token",
            vec![
                ("code", code.to_string()),
                ("grant_type", "authorization_code".to_string()),
                ("client_id", cfg.dropbox_app_key.clone()),
                ("client_secret", dropbox_app_secret()?),
                ("redirect_uri", redirect_uri.to_string()),
            ],
        ),
        SyncProviderType::Onedrive => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            vec![
                ("code", code.to_string()),
                ("client_id", cfg.onedrive_client_id.clone()),
                ("redirect_uri", redirect_uri.to_string()),
                ("grant_type", "authorization_code".to_string()),
                ("scope", "Files.ReadWrite.AppFolder offline_access".to_string()),
            ],
        ),
        _ => return Err("provider does not require oauth".to_string()),
    };

    let resp = client.post(url).form(&body).send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "oauth exchange failed: {} {}",
            status,
            String::from_utf8_lossy(&bytes)
        ));
    }
    let data: OAuthTokenResponse = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok(OAuthTokens {
        access_token: data.access_token,
        refresh_token: data.refresh_token,
        expires_at: now_ts() + data.expires_in * 1000,
    })
}

pub async fn refresh_oauth_token(
    provider: &SyncProviderType,
    cfg: &SyncConfig,
    tokens: &OAuthTokens,
) -> Result<OAuthTokens, String> {
    let client = tauri_plugin_http::reqwest::Client::new();
    let (url, body) = match provider {
        SyncProviderType::Gdrive => (
            "https://oauth2.googleapis.com/token",
            vec![
                ("refresh_token", tokens.refresh_token.clone()),
                ("client_id", gdrive_oauth_client_id().to_string()),
                ("grant_type", "refresh_token".to_string()),
            ],
        ),
        SyncProviderType::Dropbox => (
            "https://api.dropbox.com/oauth2/token",
            vec![
                ("refresh_token", tokens.refresh_token.clone()),
                ("client_id", cfg.dropbox_app_key.clone()),
                ("client_secret", dropbox_app_secret()?),
                ("grant_type", "refresh_token".to_string()),
            ],
        ),
        SyncProviderType::Onedrive => (
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            vec![
                ("refresh_token", tokens.refresh_token.clone()),
                ("client_id", cfg.onedrive_client_id.clone()),
                ("grant_type", "refresh_token".to_string()),
                ("scope", "Files.ReadWrite.AppFolder offline_access".to_string()),
            ],
        ),
        _ => return Err("provider has no oauth refresh".to_string()),
    };
    let resp = client.post(url).form(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("token refresh failed: {}", resp.status()));
    }
    let data: OAuthTokenResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(OAuthTokens {
        access_token: data.access_token,
        refresh_token: if data.refresh_token.is_empty() {
            tokens.refresh_token.clone()
        } else {
            data.refresh_token
        },
        expires_at: now_ts() + data.expires_in * 1000,
    })
}

async fn ensure_provider_access_token(provider: &SyncProviderType, cfg: &SyncConfig) -> Result<String, String> {
    let key = token_key(provider).ok_or("provider has no oauth token")?;
    let mut tokens = load_oauth_tokens(key)?.ok_or("oauth tokens missing")?;
    const REFRESH_MARGIN_MS: i64 = 5 * 60 * 1000;
    if tokens.expires_at <= now_ts() + REFRESH_MARGIN_MS {
        tokens = refresh_oauth_token(provider, cfg, &tokens).await?;
        save_oauth_tokens(key, &tokens)?;
    }
    Ok(tokens.access_token)
}

pub async fn provider_access_token(provider: &SyncProviderType, cfg: &SyncConfig) -> Result<Option<String>, String> {
    match provider {
        SyncProviderType::Gdrive | SyncProviderType::Dropbox | SyncProviderType::Onedrive => {
            Ok(Some(ensure_provider_access_token(provider, cfg).await?))
        }
        _ => Ok(None),
    }
}
