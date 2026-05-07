//! Wire-формат sync v2: `SS2` + zstd(postcard-envelope), без legacy JSON-веток.

use super::SyncPayload;
use base64::Engine as _;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

const MAGIC: &[u8] = b"SS2";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SyncWireEnvelope {
    version: u8,
    meta: crate::commands::sync::types::SyncMeta,
    tombstones: Vec<crate::models::DeletionTombstone>,
    snapshot_archive: Vec<u8>,
}

fn is_base64_icon(icon: &str) -> bool {
    icon.starts_with("data:")
}

fn data_uri_to_bytes(data_uri: &str) -> Result<Vec<u8>, String> {
    let base64_part = data_uri
        .split(',')
        .nth(1)
        .ok_or("invalid data uri")?;
    base64::engine::general_purpose::STANDARD
        .decode(base64_part)
        .map_err(|e| e.to_string())
}

fn bytes_to_data_uri(bytes: &[u8], filename: &str) -> String {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("png")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", mime, b64)
}

fn build_sync_snapshot_archive(
    data: crate::models::AppDataDoc,
    app_config: crate::models::AppConfigDoc,
) -> Result<Vec<u8>, String> {
    let mut data = data;
    let mut icons: HashMap<String, String> = HashMap::new();
    let mut idx = 0usize;

    let mut remember_icon = |raw: &str| -> String {
        if let Some((name, _)) = icons.iter().find(|(_, v)| v.as_str() == raw) {
            return name.clone();
        }
        let ext = raw
            .strip_prefix("data:image/")
            .and_then(|s| s.split(';').next())
            .unwrap_or("png")
            .replace("jpeg", "jpg");
        let name = format!("icon_{}.{}", idx, ext);
        idx += 1;
        icons.insert(name.clone(), raw.to_string());
        name
    };

    for s in &mut data.subscriptions {
        if is_base64_icon(&s.logo) {
            let name = remember_icon(&s.logo);
            s.logo = format!("icons/{}", name);
        }
    }

    let mut buf = Cursor::new(Vec::<u8>::new());
    let mut zip = ZipWriter::new(&mut buf);
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("data.json", options).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({
        "schemaVersion": 2,
        "appData": data,
        "appConfig": app_config,
    });
    zip.write_all(serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?.as_bytes())
        .map_err(|e| e.to_string())?;

    for (name, data_uri) in &icons {
        zip.start_file(format!("icons/{}", name), options)
            .map_err(|e| e.to_string())?;
        let bytes = data_uri_to_bytes(data_uri)?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

fn parse_sync_snapshot_archive(
    bytes: Vec<u8>,
) -> Result<(crate::models::AppDataDoc, crate::models::AppConfigDoc), String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;
    let mut data_json = String::new();
    {
        let mut data_file = archive.by_name("data.json").map_err(|e| e.to_string())?;
        data_file
            .read_to_string(&mut data_json)
            .map_err(|e| e.to_string())?;
    }
    let raw: serde_json::Value = serde_json::from_str(&data_json).map_err(|e| e.to_string())?;
    let app_data_val = raw
        .get("appData")
        .ok_or("sync snapshot missing appData")?
        .clone();
    let app_cfg_val = raw
        .get("appConfig")
        .ok_or("sync snapshot missing appConfig")?
        .clone();
    let mut app_data: crate::models::AppDataDoc =
        serde_json::from_value(app_data_val).map_err(|e| e.to_string())?;
    let mut app_config: crate::models::AppConfigDoc =
        serde_json::from_value(app_cfg_val).map_err(|e| e.to_string())?;
    app_config.initialized = true;

    let mut icons = HashMap::<String, String>::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        if !name.starts_with("icons/") || file.is_dir() {
            continue;
        }
        let mut raw = Vec::new();
        file.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        icons.insert(name.clone(), bytes_to_data_uri(&raw, &name));
    }
    for s in &mut app_data.subscriptions {
        if let Some(inline) = icons.get(&s.logo) {
            s.logo = inline.clone();
        }
    }
    Ok((app_data, app_config))
}

pub fn encode_sync_payload(payload: &SyncPayload) -> Result<Vec<u8>, String> {
    let snapshot_archive = build_sync_snapshot_archive(payload.data.clone(), payload.app_config.clone())?;
    let envelope = SyncWireEnvelope {
        version: 2,
        meta: payload.meta.clone(),
        tombstones: payload.tombstones.clone(),
        snapshot_archive,
    };
    let raw = postcard::to_allocvec(&envelope).map_err(|e| e.to_string())?;
    let compressed = zstd::encode_all(raw.as_slice(), 3).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(MAGIC.len() + compressed.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&compressed);
    Ok(out)
}

pub fn decode_sync_payload(bytes: &[u8]) -> Result<SyncPayload, String> {
    if !bytes.starts_with(MAGIC) {
        return Err("unsupported sync payload format".to_string());
    }
    let raw = zstd::decode_all(&bytes[MAGIC.len()..]).map_err(|e| e.to_string())?;
    let envelope: SyncWireEnvelope = postcard::from_bytes(&raw).map_err(|e| e.to_string())?;
    if envelope.version != 2 {
        return Err("unsupported sync payload version".to_string());
    }
    let (data, app_config) = parse_sync_snapshot_archive(envelope.snapshot_archive)?;
    Ok(SyncPayload {
        data,
        app_config,
        meta: envelope.meta,
        tombstones: envelope.tombstones,
    })
}
