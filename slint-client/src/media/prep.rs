//! Attachment preparation (port of send-media.ts): image compression,
//! thumbnails, temp-file staging and MIME/extension maps.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine;
use image::DynamicImage;
use image::codecs::jpeg::JpegEncoder;

/// Standard / HD image sends: longest edge and JPEG quality (never upscaled).
pub const STANDARD_EDGE: u32 = 1600;
pub const STANDARD_QUALITY: u8 = 75;
pub const HD_EDGE: u32 = 3000;
pub const HD_QUALITY: u8 = 92;
/// Inline thumbnails: longest edge and JPEG quality.
pub const THUMB_EDGE: u32 = 200;
pub const THUMB_QUALITY: u8 = 60;

pub fn decode(path: &Path) -> anyhow::Result<DynamicImage> {
    Ok(image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?)
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality).encode_image(&img.to_rgb8())?;
    Ok(out)
}

/// Base64 JPEG thumbnail (no `data:` prefix), longest edge ≤ 200px, q60.
pub fn thumbnail_b64(img: &DynamicImage) -> anyhow::Result<String> {
    let t = if img.width() > THUMB_EDGE || img.height() > THUMB_EDGE {
        img.thumbnail(THUMB_EDGE, THUMB_EDGE)
    } else {
        img.clone()
    };
    Ok(base64::engine::general_purpose::STANDARD.encode(encode_jpeg(&t, THUMB_QUALITY)?))
}

/// Re-encode an image for sending. Returns (staged jpeg path, thumbnail).
pub fn compress_image(src: &Path, hd: bool) -> anyhow::Result<(PathBuf, String)> {
    let img = decode(src)?;
    let edge = if hd { HD_EDGE } else { STANDARD_EDGE };
    let img = if img.width() > edge || img.height() > edge {
        img.resize(edge, edge, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let bytes = encode_jpeg(&img, if hd { HD_QUALITY } else { STANDARD_QUALITY })?;
    let path = stage_path("jpg");
    std::fs::write(&path, bytes)?;
    Ok((path, thumbnail_b64(&img)?))
}

/// A unique temp path for a staged upload (`wa-send-<ts>-<n>.<ext>`).
pub fn stage_path(ext: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("wa-send-{ts}-{n}.{ext}"))
}

pub fn is_gif(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("gif"))
}

pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .is_some_and(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"))
}

/// MIME for a document by extension (port of mimeForPath); None lets the
/// backend decide.
pub fn mime_for_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    Some(match ext.as_str() {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "zip" => "application/zip",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => return None,
    })
}

/// Best-effort removal of a staged temp file after sending.
pub fn unstage(path: &Path) {
    if path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("wa-send-"))
    {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compresses_and_thumbnails() {
        let src = stage_path("png");
        image::RgbImage::from_pixel(4000, 2000, image::Rgb([200, 10, 10]))
            .save(&src)
            .unwrap();
        let (out, thumb) = compress_image(&src, false).unwrap();
        let img = decode(&out).unwrap();
        assert_eq!((img.width(), img.height()), (1600, 800));
        assert!(!thumb.is_empty());
        unstage(&src);
        unstage(&out);
    }

    #[test]
    fn mime_map() {
        assert_eq!(mime_for_path(Path::new("a.PDF")), Some("application/pdf"));
        assert_eq!(mime_for_path(Path::new("a.xyz")), None);
    }
}
