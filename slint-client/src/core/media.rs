//! Media download / save / send. Fetches + decrypts a media payload to a
//! content-addressed file in the app-data cache and returns its path; the UI
//! decodes it straight from disk (images) or hands it to the OS (video, audio,
//! documents), so large media never sits in UI memory.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use serde::Deserialize;
use tokio::sync::Semaphore;
use whatsapp_rust::UploadOptions;
use whatsapp_rust::download::DownloadParams;
use whatsapp_rust::media::{self, AudioOptions, DocumentOptions, ImageOptions, VideoOptions};
use whatsapp_rust::wacore::download::MediaType;

use crate::core::api::{Api, parse_jid};
use crate::core::dto::{MediaDescriptorDto, SendResultDto};
use crate::core::error::{ApiError, ApiResult};
use crate::core::paths;
use crate::core::session::Session;

/// The download descriptor (same shape the bridge produces).
pub type MediaDescriptorIn = MediaDescriptorDto;

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

/// Extension of a document's display name, when it is a plain, short one.
fn name_ext(file_name: Option<&str>) -> Option<String> {
    let name = file_name?.rsplit(['/', '\\']).next()?;
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() || ext.is_empty() || ext.len() > 10 {
        return None;
    }
    ext.chars()
        .all(|c| c.is_ascii_alphanumeric())
        .then(|| ext.to_ascii_lowercase())
}

/// File extension for a cached payload. Documents keep their own extension
/// (from the display name, else the mimetype) so the OS can open the cached copy
/// with the right app instead of seeing an opaque `.bin`.
fn ext_for(mimetype: Option<&str>, media_type: &str, file_name: Option<&str>) -> String {
    if media_type == "document" {
        if let Some(ext) = name_ext(file_name) {
            return ext;
        }
    }
    let mime = mimetype.map(|m| m.split(';').next().unwrap_or(m).trim());
    let ext = match mime {
        Some("image/png") => "png",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        Some(m) if m.starts_with("image/") => "jpg",
        Some("video/webm") => "webm",
        Some("video/quicktime") => "mov",
        Some(m) if m.starts_with("video/") => "mp4",
        Some("audio/ogg") => "ogg",
        Some("audio/mpeg") => "mp3",
        Some(m) if m.starts_with("audio/") => "m4a",
        Some("application/pdf") => "pdf",
        Some("application/msword") => "doc",
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document") => "docx",
        Some("application/vnd.ms-excel") => "xls",
        Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet") => "xlsx",
        Some("application/vnd.ms-powerpoint") => "ppt",
        Some("application/vnd.openxmlformats-officedocument.presentationml.presentation") => "pptx",
        Some("application/zip") | Some("application/x-zip-compressed") => "zip",
        Some("application/json") => "json",
        Some("text/plain") => "txt",
        Some("text/csv") => "csv",
        Some("text/html") => "html",
        _ => match media_type {
            "video" => "mp4",
            "audio" => "ogg",
            "sticker" => "webp",
            "document" => "bin",
            _ => "jpg",
        },
    };
    ext.to_string()
}

/// Max media downloads in flight at once. Scrolling a media-heavy chat mounts
/// many auto-loading bubbles; unbounded, each one holds a connection, a decrypt
/// buffer and a blocking thread at the same time.
const MAX_CONCURRENT_DOWNLOADS: usize = 3;

fn download_permits() -> &'static Semaphore {
    static PERMITS: OnceLock<Semaphore> = OnceLock::new();
    PERMITS.get_or_init(|| Semaphore::new(MAX_CONCURRENT_DOWNLOADS))
}

/// Ensure a media payload is in the content-addressed cache and return its path.
///
/// Streams download → decrypt → file with constant memory (the library's
/// `download_from_params_to_writer`), instead of buffering the whole payload —
/// a large video used to sit in RAM in full (plus its encrypted copy). Each call
/// writes its own `.part` file and renames it into place, so concurrent
/// requests for the same item can't corrupt each other, and a crash mid-write
/// never leaves a truncated file that a later cache hit would serve.
async fn ensure_cached(
    session: &Session,
    descriptor: MediaDescriptorIn,
    mimetype: Option<&str>,
    file_name: Option<&str>,
) -> ApiResult<PathBuf> {
    let media_key = b64_decode(&descriptor.media_key)?;
    let file_sha256 = b64_decode(&descriptor.file_sha256)?;
    let file_enc_sha256 = b64_decode(&descriptor.file_enc_sha256)?;

    // Content-addressed filename (url-safe so it's filesystem-legal).
    let stem = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&file_sha256);
    let ext = ext_for(mimetype, &descriptor.media_type, file_name);

    let dir = paths::media_dir();
    let path = dir.join(format!("{stem}.{ext}"));
    if path.exists() {
        return Ok(path);
    }

    let _permit = download_permits()
        .acquire()
        .await
        .map_err(ApiError::library)?;
    // Another request may have finished this item while we waited.
    if path.exists() {
        return Ok(path);
    }

    static PART_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = PART_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!("{stem}.{ext}.{seq}.part"));
    let file = std::fs::File::create(&tmp).map_err(ApiError::library)?;

    let params = DownloadParams::encrypted(
        descriptor.direct_path,
        &media_key,
        &file_sha256,
        &file_enc_sha256,
        descriptor.file_length,
        media_type_of(&descriptor.media_type),
    );
    let result = session
        .client
        .download_from_params_to_writer(&params, file)
        .await;
    match result {
        Ok(file) => {
            // Flush and close the handle before renaming (required on Windows).
            let synced = file.sync_all();
            drop(file);
            if let Err(e) = synced {
                let _ = std::fs::remove_file(&tmp);
                return Err(ApiError::library(e));
            }
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(ApiError::library(e));
        }
    }
    if let Err(e) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        // A concurrent download of the same item won the rename; use its file.
        if !path.exists() {
            return Err(ApiError::library(e));
        }
    }
    Ok(path)
}

impl Api {
    /// Download+decrypt a media payload into the cache and return its absolute path.
    /// Idempotent: a content-addressed cache hit returns immediately.
    pub async fn download_media(
        &self,
        descriptor: MediaDescriptorIn,
        mimetype: Option<String>,
        file_name: Option<String>,
    ) -> ApiResult<String> {
        let (_, session) = self.mgr.session(None).await?;
        let path = ensure_cached(
            &session,
            descriptor,
            mimetype.as_deref(),
            file_name.as_deref(),
        )
        .await?;
        Ok(path.to_string_lossy().into_owned())
    }
}

/// Sanitize a sender-supplied display name into a single, safe filename
/// component so the copy always lands directly inside Downloads.
///
/// Strips any directory part (both separators), and everything Windows treats
/// specially: a drive prefix (`C:evil.dll` is drive-relative and would resolve
/// outside Downloads), `:` (alternate data streams), reserved characters,
/// control characters, trailing dots/spaces and reserved device names.
fn safe_file_name(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let mut out: String = base
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    out = out.trim().trim_end_matches(['.', ' ']).to_string();
    if out.is_empty() || out.chars().all(|c| c == '.' || c == '_') {
        return None;
    }
    let stem = out
        .split('.')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        out.insert(0, '_');
    }
    // Keep names well under path-length limits (char-boundary safe).
    if out.chars().count() > 180 {
        let ext = std::path::Path::new(&out)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .filter(|e| e.len() <= 11)
            .unwrap_or_default();
        let keep: String = out.chars().take(180 - ext.chars().count()).collect();
        out = format!("{keep}{ext}");
    }
    Some(out)
}

impl Api {
    /// Copy a media payload into the user's OS Downloads folder, downloading and
    /// decrypting it first if it isn't already in the content-addressed cache
    /// (shares `download_media`'s cache, so an already-viewed item copies instantly).
    /// Returns the absolute path of the saved file. If a file of the same name
    /// already exists, a numeric suffix is appended so we never overwrite.
    pub async fn save_media_to_downloads(
        &self,
        descriptor: MediaDescriptorIn,
        mimetype: Option<String>,
        filename: Option<String>,
    ) -> ApiResult<String> {
        let (_, session) = self.mgr.session(None).await?;

        // Display-name fallback: the cache file's own name (content hash + ext).
        let ext = ext_for(
            mimetype.as_deref(),
            &descriptor.media_type,
            filename.as_deref(),
        );
        let file_sha256 = b64_decode(&descriptor.file_sha256)?;
        let stem = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&file_sha256);

        // Shares download_media's cache, so an already-viewed item copies instantly.
        let cached = ensure_cached(
            &session,
            descriptor,
            mimetype.as_deref(),
            filename.as_deref(),
        )
        .await?;

        let downloads =
            paths::download_dir().ok_or_else(|| ApiError::library("no downloads dir"))?;
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

        // Belt and braces: the sanitized name must resolve directly inside Downloads.
        if dest.parent() != Some(downloads.as_path()) {
            return Err(ApiError::library("refusing to save outside Downloads"));
        }
        std::fs::copy(&cached, &dest).map_err(ApiError::library)?;
        Ok(dest.to_string_lossy().into_owned())
    }
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
    /// Video only: loop silently inline like a GIF. The payload must still be
    /// an MP4 — WhatsApp clients can't play GIF bytes inside a video message.
    pub gif_playback: Option<bool>,
}

impl Api {
    /// Upload a local file and send it as a media message. The bytes are read from a
    /// path (not marshalled over IPC) so large videos never cross JS memory; the
    /// frontend writes in-memory blobs (pastes, recordings) to a temp file first.
    pub async fn send_media(
        &self,
        jid: String,
        path: String,
        media_type: String,
        options: Option<MediaSendOptionsIn>,
    ) -> ApiResult<SendResultDto> {
        let (id, session) = self.mgr.session(None).await?;
        let to = parse_jid(&jid)?;
        let opts = options.unwrap_or_default();
        // WhatsApp clients can't play GIF bytes inside a video message.
        if media_type == "video"
            && opts
                .mimetype
                .as_deref()
                .is_some_and(|m| m.starts_with("image/gif"))
        {
            return Err(ApiError::library(
                "GIF bytes can't be sent as a video; send an MP4 with gifPlayback",
            ));
        }

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
                // Animated GIFs are sent as MP4 videos with the gif-playback flag so
                // they auto-loop inline (WhatsApp behaviour).
                let gif_playback = opts.gif_playback.filter(|g| *g);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_file_name_stays_a_single_component() {
        assert_eq!(safe_file_name("report.pdf").as_deref(), Some("report.pdf"));
        assert_eq!(safe_file_name("../../x.dll").as_deref(), Some("x.dll"));
        assert_eq!(
            safe_file_name("C:\\Windows\\x.dll").as_deref(),
            Some("x.dll")
        );
        assert_eq!(safe_file_name("C:x.dll").as_deref(), Some("C_x.dll"));
        assert_eq!(
            safe_file_name("a.txt:stream").as_deref(),
            Some("a.txt_stream")
        );
        assert_eq!(safe_file_name("CON.txt").as_deref(), Some("_CON.txt"));
        assert_eq!(safe_file_name("com1").as_deref(), Some("_com1"));
        assert_eq!(safe_file_name("name. . ").as_deref(), Some("name"));
        assert_eq!(safe_file_name(".."), None);
        assert_eq!(safe_file_name("  "), None);
        let long = format!("{}.pdf", "a".repeat(400));
        let s = safe_file_name(&long).unwrap();
        assert_eq!(s.chars().count(), 180);
        assert!(s.ends_with(".pdf"));
    }

    #[test]
    fn documents_keep_their_extension() {
        assert_eq!(ext_for(None, "document", Some("Q3.XLSX")), "xlsx");
        assert_eq!(ext_for(Some("application/pdf"), "document", None), "pdf");
        assert_eq!(
            ext_for(
                Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                "document",
                Some("noext")
            ),
            "docx"
        );
        assert_eq!(ext_for(None, "document", Some("weird.e x e")), "bin");
        assert_eq!(ext_for(Some("image/png"), "image", Some("x.exe")), "png");
    }
}
