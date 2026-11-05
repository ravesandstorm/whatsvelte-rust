//! Image decoding for the UI. Everything here runs on worker threads and
//! produces `SharedPixelBuffer`s (which are `Send`); the UI thread only wraps
//! them in `slint::Image`, so decoding never stalls rendering.

use std::collections::{HashMap, VecDeque};
use std::path::Path;

use base64::Engine;
use slint::{Rgba8Pixel, SharedPixelBuffer};

pub type Pixels = SharedPixelBuffer<Rgba8Pixel>;

/// Longest edge kept for in-bubble images; the viewer decodes the original.
pub const BUBBLE_MAX_EDGE: u32 = 560;

fn to_pixels(img: image::DynamicImage, max_edge: u32) -> Pixels {
    let img = if img.width() > max_edge || img.height() > max_edge {
        img.thumbnail(max_edge, max_edge)
    } else {
        img
    };
    let rgba = img.into_rgba8();
    SharedPixelBuffer::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height())
}

/// Decode an inline base64 JPEG thumbnail.
pub fn decode_b64(b64: &str) -> Option<Pixels> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    Some(to_pixels(img, BUBBLE_MAX_EDGE))
}

/// Decode a cached media file, downscaled to `max_edge`.
pub fn decode_file(path: &Path, max_edge: u32) -> Option<Pixels> {
    let img = image::ImageReader::open(path).ok()?.with_guessed_format().ok()?.decode().ok()?;
    Some(to_pixels(img, max_edge))
}

/// Build a small base64 JPEG thumbnail for an outgoing image.
pub fn jpeg_thumbnail(path: &Path) -> Option<String> {
    let img = image::ImageReader::open(path).ok()?.with_guessed_format().ok()?.decode().ok()?;
    let thumb = img.thumbnail(96, 96).into_rgb8();
    let mut out = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut out, image::ImageFormat::Jpeg).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(out.into_inner()))
}

/// Bounded FIFO cache of decoded images keyed by message id (+ variant).
pub struct ImageCache {
    map: HashMap<String, Pixels>,
    order: VecDeque<String>,
    cap: usize,
}

impl ImageCache {
    pub fn new(cap: usize) -> Self {
        Self { map: HashMap::new(), order: VecDeque::new(), cap }
    }

    pub fn get(&self, key: &str) -> Option<Pixels> {
        self.map.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, px: Pixels) {
        if self.map.insert(key.clone(), px).is_none() {
            self.order.push_back(key);
            while self.order.len() > self.cap {
                if let Some(old) = self.order.pop_front() {
                    self.map.remove(&old);
                }
            }
        }
    }
}
