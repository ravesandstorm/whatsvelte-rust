//! Image decoding for the UI. Everything here runs on worker threads and
//! produces `SharedPixelBuffer`s (which are `Send`); the UI thread only wraps
//! them in `slint::Image`, so decoding never stalls rendering.

use std::collections::HashMap;
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
    let img = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    Some(to_pixels(img, max_edge))
}

/// LRU cache of decoded images keyed by message id (+ variant), bounded by the
/// bytes of pixel data it holds rather than by entry count: a full photo is
/// ~100x the size of a thumbnail.
pub struct ImageCache {
    map: HashMap<String, (Pixels, u64)>,
    bytes: usize,
    budget: usize,
    clock: u64,
}

impl ImageCache {
    pub fn new(budget: usize) -> Self {
        Self {
            map: HashMap::new(),
            bytes: 0,
            budget,
            clock: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<Pixels> {
        self.clock += 1;
        let clock = self.clock;
        self.map.get_mut(key).map(|(px, used)| {
            *used = clock;
            px.clone()
        })
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn insert(&mut self, key: String, px: Pixels) {
        self.clock += 1;
        self.bytes += px.as_bytes().len();
        if let Some((old, _)) = self.map.insert(key, (px, self.clock)) {
            self.bytes -= old.as_bytes().len();
        }
        while self.bytes > self.budget && self.map.len() > 1 {
            let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            if let Some((old, _)) = self.map.remove(&oldest) {
                self.bytes -= old.as_bytes().len();
            }
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.bytes = 0;
    }

    #[cfg(test)]
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px() -> Pixels {
        SharedPixelBuffer::new(10, 10) // 400 bytes
    }

    #[test]
    fn evicts_least_recently_used_over_budget() {
        let mut c = ImageCache::new(1000);
        c.insert("a".into(), px());
        c.insert("b".into(), px());
        assert!(c.get("a").is_some());
        c.insert("c".into(), px());
        assert!(c.contains("a") && c.contains("c"));
        assert!(!c.contains("b"));
        assert_eq!(c.bytes(), 800);
    }

    #[test]
    fn replacing_a_key_does_not_leak_budget() {
        let mut c = ImageCache::new(1000);
        c.insert("a".into(), px());
        c.insert("a".into(), px());
        assert_eq!(c.bytes(), 400);
    }
}
