use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncProviderType {
    Icloud,
    Gdrive,
    Dropbox,
    Onedrive,
    Webdav,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub provider: Option<SyncProviderType>,
    pub enabled: bool,
    pub last_synced: i64,
    pub syncing: bool,
    pub error: Option<String>,
    pub remote_updated_at: i64,
    pub local_updated_at: i64,
    pub pending_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub provider: Option<SyncProviderType>,
    pub enabled: bool,
    pub last_synced: i64,
    pub local_updated_at: i64,
    pub device_id: String,
    pub dropbox_app_key: String,
    pub onedrive_client_id: String,
    pub webdav_url: String,
    pub webdav_username: String,
    pub remote_revision: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: None,
            enabled: false,
            last_synced: 0,
            local_updated_at: 0,
            device_id: String::new(),
            dropbox_app_key: String::new(),
            onedrive_client_id: String::new(),
            webdav_url: String::new(),
            webdav_username: String::new(),
            remote_revision: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMeta {
    pub last_synced_at: i64,
    pub updated_at: i64,
    pub device_id: String,
    pub revision: Option<String>,
    pub schema_version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayload {
    pub data: crate::models::AppDataDoc,
    pub app_config: crate::models::AppConfigDoc,
    pub meta: SyncMeta,
    #[serde(default)]
    pub tombstones: Vec<crate::models::DeletionTombstone>,
}
