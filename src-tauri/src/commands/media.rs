//! Media download. Fetches + decrypts a media payload to a content-addressed
//! file in the app-data cache and returns its path; the frontend renders it via
//! Tauri's asset protocol (`convertFileSrc`), bypassing the IPC bridge so large
//! media never crosses JS memory.

use std::sync::Arc;

use base64::Engine;
use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
use whatsapp_rust::download::DownloadParams;
use whatsapp_rust::media::{self, AudioOptions, DocumentOptions, ImageOptions, VideoOptions};
use whatsapp_rust::wacore::download::MediaType;
use whatsapp_rust::{Jid, UploadOptions};

use crate::dto::SendResultDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

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

/// Sanitize a user-supplied display name into a single, safe filename component
/// (no path separators, no traversal) so the copy lands inside Downloads.
fn safe_file_name(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    if base.is_empty() || base == "." || base == ".." {
        return None;
    }
    Some(base.to_string())
}

/// Copy a media payload into the user's OS Downloads folder, downloading and
/// decrypting it first if it isn't already in the content-addressed cache
/// (shares `download_media`'s cache, so an already-viewed item copies instantly).
/// Returns the absolute path of the saved file. If a file of the same name
/// already exists, a numeric suffix is appended so we never overwrite.
#[tauri::command]
pub async fn save_media_to_downloads(
    descriptor: MediaDescriptorIn,
    mimetype: Option<String>,
    filename: Option<String>,
    session_id: Option<String>,
    app: AppHandle,
    mgr: Mgr<'_>,
) -> ApiResult<String> {
    let (_, session) = mgr.session(session_id).await?;

    let media_key = b64_decode(&descriptor.media_key)?;
    let file_sha256 = b64_decode(&descriptor.file_sha256)?;
    let file_enc_sha256 = b64_decode(&descriptor.file_enc_sha256)?;

    let stem = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&file_sha256);
    let ext = ext_for(mimetype.as_deref(), &descriptor.media_type);

    // Ensure the file exists in the shared cache (mirrors download_media).
    let mut cache_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ApiError::library(format!("no app data dir: {e}")))?;
    cache_dir.push("media");
    std::fs::create_dir_all(&cache_dir).map_err(ApiError::library)?;
    let cached = cache_dir.join(format!("{stem}.{ext}"));

    if !cached.exists() {
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
        let tmp = cached.with_extension(format!("{ext}.part"));
        std::fs::write(&tmp, &bytes).map_err(ApiError::library)?;
        std::fs::rename(&tmp, &cached).map_err(ApiError::library)?;
    }

    let downloads = app
        .path()
        .download_dir()
        .map_err(|e| ApiError::library(format!("no downloads dir: {e}")))?;
    std::fs::create_dir_all(&downloads).map_err(ApiError::library)?;

    // Prefer the message's display name; fall back to the content hash + ext.
    let base = filename
        .as_deref()
        .and_then(safe_file_name)
        .unwrap_or_else(|| format!("{stem}.{ext}"));

    // Don't clobber an existing file: "name.ext" -> "name (1).ext", etc.
    let mut dest = downloads.join(&base);
    if dest.exists() {
        let path = std::path::Path::new(&base);
        let file_stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| base.clone());
        let suffix = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        for n in 1.. {
            let candidate = downloads.join(format!("{file_stem} ({n}){suffix}"));
            if !candidate.exists() {
                dest = candidate;
                break;
            }
        }
    }

    std::fs::copy(&cached, &dest).map_err(ApiError::library)?;
    Ok(dest.to_string_lossy().into_owned())
}

/// Per-type send options from the UI. All optional; the matching builder fills
/// sensible defaults (mimetype, etc.). Mirrors the frontend `MediaSendOptions`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSendOptionsIn {
    pub caption: Option<String>,
    pub mimetype: Option<String>,
    /// Document display name.
    pub file_name: Option<String>,
    /// Audio/video length.
    pub duration_secs: Option<u32>,
    /// Voice-note flag.
    pub ptt: Option<bool>,
    /// base64 JPEG inline thumbnail (image/video).
    pub jpeg_thumbnail: Option<String>,
}

/// Upload a local file and send it as a media message. The bytes are read from a
/// path (not marshalled over IPC) so large videos never cross JS memory; the
/// frontend writes in-memory blobs (pastes, recordings) to a temp file first.
#[tauri::command]
pub async fn send_media(
    jid: String,
    path: String,
    media_type: String,
    options: Option<MediaSendOptionsIn>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<SendResultDto> {
    let (id, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;
    let opts = options.unwrap_or_default();

    let bytes = std::fs::read(&path).map_err(ApiError::library)?;
    let thumb = match opts.jpeg_thumbnail.as_deref() {
        Some(s) => Some(b64_decode(s)?),
        None => None,
    };

    let upload = session
        .client
        .upload(bytes, media_type_of(&media_type), UploadOptions::default())
        .await
        .map_err(ApiError::library)?;

    let msg = match media_type.as_str() {
        "video" => {
            // Animated GIFs are sent as videos with the gif-playback flag so they
            // auto-loop inline (WhatsApp behaviour); regular videos leave it unset.
            let gif_playback = match opts.mimetype.as_deref() {
                Some(m) if m.starts_with("image/gif") => Some(true),
                _ => None,
            };
            media::video_message(
                upload,
                VideoOptions {
                    caption: opts.caption,
                    mimetype: opts.mimetype,
                    jpeg_thumbnail: thumb,
                    duration_seconds: opts.duration_secs,
                    gif_playback,
                },
            )
        }
        "audio" => media::audio_message(
            upload,
            AudioOptions {
                mimetype: opts.mimetype,
                duration_seconds: opts.duration_secs,
                ptt: opts.ptt,
                waveform: None,
            },
        ),
        "document" => media::document_message(
            upload,
            DocumentOptions {
                mimetype: opts.mimetype,
                file_name: opts.file_name,
                title: None,
                caption: opts.caption,
                page_count: None,
                jpeg_thumbnail: thumb,
            },
        ),
        _ => media::image_message(
            upload,
            ImageOptions {
                caption: opts.caption,
                mimetype: opts.mimetype,
                jpeg_thumbnail: thumb,
            },
        ),
    };

    let result = session
        .client
        .send_message(to, msg)
        .await
        .map_err(ApiError::library)?;

    Ok(SendResultDto {
        session_id: id,
        message_id: result.message_id,
        to: result.to.to_string(),
    })
}
