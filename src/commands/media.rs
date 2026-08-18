//! Media operations for WhatsApp Dioxus.

use base64::Engine;
use serde::Deserialize;
use whatsapp_rust::download::DownloadParams;
use whatsapp_rust::media::{self, AudioOptions, DocumentOptions, ImageOptions, VideoOptions};
use whatsapp_rust::wacore::download::MediaType;
use whatsapp_rust::{Jid, UploadOptions};

use crate::dto::SendResultDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

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
        .map_err(ApiError::library)
}

fn media_type_of(s: &str) -> MediaType {
    match s {
        "image" => MediaType::Image,
        "video" => MediaType::Video,
        "audio" => MediaType::Audio,
        "document" => MediaType::Document,
        "sticker" => MediaType::Sticker,
        _ => MediaType::Document,
    }
}

fn ext_for(mime: Option<&str>, kind: &str) -> &'static str {
    match mime {
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        Some("video/mp4") => "mp4",
        Some("audio/ogg") | Some("audio/ogg; codecs=opus") => "ogg",
        Some("audio/mp4") | Some("audio/m4a") => "m4a",
        Some("audio/mpeg") => "mp3",
        Some("application/pdf") => "pdf",
        _ => match kind {
            "image" => "jpg",
            "video" => "mp4",
            "audio" => "ogg",
            "sticker" => "webp",
            "document" => "bin",
            _ => "jpg",
        },
    }
}

pub async fn download_media(
    descriptor: MediaDescriptorIn,
    mimetype: Option<String>,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<String> {
    let (_, session) = mgr.session(session_id).await?;

    let media_key = b64_decode(&descriptor.media_key)?;
    let file_sha256 = b64_decode(&descriptor.file_sha256)?;
    let file_enc_sha256 = b64_decode(&descriptor.file_enc_sha256)?;

    let stem = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&file_sha256);
    let ext = ext_for(mimetype.as_deref(), &descriptor.media_type);

    let mut dir = std::env::temp_dir();
    dir.push("whatdioxus_media");
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

    let tmp = path.with_extension(format!("{ext}.part"));
    std::fs::write(&tmp, &bytes).map_err(ApiError::library)?;
    std::fs::rename(&tmp, &path).map_err(ApiError::library)?;

    Ok(path.to_string_lossy().into_owned())
}

fn safe_file_name(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    if base.is_empty() || base == "." || base == ".." {
        return None;
    }
    Some(base.to_string())
}

pub async fn save_media_to_downloads(
    descriptor: MediaDescriptorIn,
    mimetype: Option<String>,
    filename: Option<String>,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<String> {
    let cached_path_str = download_media(descriptor, mimetype.clone(), session_id, mgr).await?;
    let cached_path = std::path::Path::new(&cached_path_str);

    let downloads_dir = dirs::download_dir()
        .or_else(dirs::desktop_dir)
        .ok_or_else(|| ApiError::library("could not resolve OS Downloads directory"))?;

    let default_name = cached_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("media");

    let clean_name = filename
        .as_deref()
        .and_then(safe_file_name)
        .unwrap_or_else(|| default_name.to_string());

    let target_path = downloads_dir.join(&clean_name);
    std::fs::copy(cached_path, &target_path).map_err(ApiError::library)?;
    Ok(target_path.to_string_lossy().into_owned())
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSendOptionsIn {
    pub caption: Option<String>,
    pub filename: Option<String>,
    pub mimetype: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<u32>,
    pub ptt: Option<bool>,
    pub jpeg_thumbnail: Option<String>,
}

pub async fn send_media(
    jid: String,
    path: String,
    media_type: String,
    options: Option<MediaSendOptionsIn>,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<SendResultDto> {
    let (id, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;
    let file_path = std::path::Path::new(&path);

    if !file_path.exists() {
        return Err(ApiError::library(format!("file not found: {path}")));
    }

    let file_bytes = std::fs::read(file_path).map_err(ApiError::library)?;
    let opts = options.unwrap_or_default();
    let media_enum = media_type_of(&media_type);

    let uploaded = session
        .client
        .upload(file_bytes, media_enum, UploadOptions::default())
        .await
        .map_err(ApiError::library)?;

    let jpeg_thumbnail = match opts.jpeg_thumbnail {
        Some(ref b64) => Some(b64_decode(b64)?),
        None => None,
    };

    let msg = match media_enum {
        MediaType::Image => media::image_message(
            uploaded,
            ImageOptions {
                caption: opts.caption,
                mimetype: opts.mimetype,
                jpeg_thumbnail,
                ..Default::default()
            },
        ),
        MediaType::Video => media::video_message(
            uploaded,
            VideoOptions {
                caption: opts.caption,
                mimetype: opts.mimetype,
                duration_seconds: opts.duration_secs,
                jpeg_thumbnail,
                ..Default::default()
            },
        ),
        MediaType::Audio => media::audio_message(
            uploaded,
            AudioOptions {
                mimetype: opts.mimetype,
                duration_seconds: opts.duration_secs,
                ptt: opts.ptt,
                ..Default::default()
            },
        ),
        MediaType::Document => media::document_message(
            uploaded,
            DocumentOptions {
                caption: opts.caption,
                file_name: opts.filename,
                mimetype: opts.mimetype,
                ..Default::default()
            },
        ),
        MediaType::Sticker => media::image_message(uploaded, Default::default()),
        _ => return Err(ApiError::library(format!("unsupported media type: {media_type}"))),
    };

    let sent = session
        .client
        .send_message(to, msg)
        .await
        .map_err(ApiError::library)?;

    Ok(SendResultDto {
        session_id: id,
        message_id: sent.message_id,
        to: jid,
    })
}
