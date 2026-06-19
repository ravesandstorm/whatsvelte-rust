//! Media download. Fetches + decrypts a media payload to a content-addressed
//! file in the app-data cache and returns its path; the frontend renders it via
//! Tauri's asset protocol (`convertFileSrc`), bypassing the IPC bridge so large
//! media never crosses JS memory.

use std::sync::Arc;

use base64::Engine;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
use whatsapp_rust::download::DownloadParams;
use whatsapp_rust::wacore::download::MediaType;

use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

/// The base64-encoded download descriptor sent from the UI (mirrors `MediaDescriptorDto`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDescriptorIn {
    pub direct_path: String,
    pub media_key: String,
    pub file_sha256: String,
    pub file_enc_sha256: String,
    pub file_length: u64,
    pub media_type: String,
}

fn b64_decode(s: &str) -> ApiResult<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| ApiError::library(format!("bad base64: {e}")))
}

fn media_type_of(s: &str) -> MediaType {
    match s {
        "video" => MediaType::Video,
        "audio" => MediaType::Audio,
        "document" => MediaType::Document,
        "sticker" => MediaType::Sticker,
        _ => MediaType::Image,
    }
}

fn ext_for(mimetype: Option<&str>, media_type: &str) -> &'static str {
    match mimetype {
        Some(m) if m.starts_with("image/png") => "png",
        Some(m) if m.starts_with("image/webp") => "webp",
        Some(m) if m.starts_with("image/gif") => "gif",
        Some(m) if m.starts_with("image/") => "jpg",
        Some(m) if m.starts_with("video/") => "mp4",
        Some(m) if m.starts_with("audio/ogg") => "ogg",
        Some(m) if m.starts_with("audio/mpeg") => "mp3",
        Some(m) if m.starts_with("audio/") => "m4a",
        Some("application/pdf") => "pdf",
        _ => match media_type {
            "video" => "mp4",
            "audio" => "ogg",
            "sticker" => "webp",
            "document" => "bin",
            _ => "jpg",
        },
    }
}

/// Download+decrypt a media payload into the cache and return its absolute path.
/// Idempotent: a content-addressed cache hit returns immediately.
#[tauri::command]
pub async fn download_media(
    descriptor: MediaDescriptorIn,
    mimetype: Option<String>,
    session_id: Option<String>,
    app: AppHandle,
    mgr: Mgr<'_>,
) -> ApiResult<String> {
    let (_, session) = mgr.session(session_id).await?;

    let media_key = b64_decode(&descriptor.media_key)?;
    let file_sha256 = b64_decode(&descriptor.file_sha256)?;
    let file_enc_sha256 = b64_decode(&descriptor.file_enc_sha256)?;

    // Content-addressed filename (url-safe so it's filesystem-legal).
    let stem = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&file_sha256);
    let ext = ext_for(mimetype.as_deref(), &descriptor.media_type);

    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ApiError::library(format!("no app data dir: {e}")))?;
    dir.push("media");
    std::fs::create_dir_all(&dir).map_err(ApiError::library)?;
    let path = dir.join(format!("{stem}.{ext}"));

    if path.exists() {
        return Ok(path.to_string_lossy().into_owned());
    }

    let params = DownloadParams::encrypted(
        descriptor.direct_path,
        &media_key,
        &file_sha256,
        &file_enc_sha256,
        descriptor.file_length,
        media_type_of(&descriptor.media_type),
    );

    let bytes = session
        .client
        .download_from_params(&params)
        .await
        .map_err(ApiError::library)?;

    // Write to a temp sibling then rename, so a crash mid-write can't leave a
    // truncated file that a future cache-hit would serve.
    let tmp = path.with_extension(format!("{ext}.part"));
    std::fs::write(&tmp, &bytes).map_err(ApiError::library)?;
    std::fs::rename(&tmp, &path).map_err(ApiError::library)?;

    Ok(path.to_string_lossy().into_owned())
}
