//! User preferences (port of settings/theme/zoom stores + emoji recents).
//! Stored as `<app-data>/settings.json`, independent of the linked account.

use serde::{Deserialize, Serialize};

use crate::core::paths;

pub const ZOOM_MIN: f32 = 0.5;
pub const ZOOM_MAX: f32 = 2.0;
pub const ZOOM_STEP: f32 = 0.1;
pub const EMOJI_RECENT_MAX: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "light" | "dark" | "system" (the Svelte UI defaulted to dark).
    pub theme: String,
    /// UI zoom factor, 0.5 ..= 2.0 in 0.1 steps.
    pub zoom: f32,
    /// Enter sends (Shift+Enter newline) vs Enter inserts a newline.
    pub enter_to_send: bool,
    /// Emit read receipts for messages scrolled into view.
    pub send_read_receipts: bool,
    /// Auto-download images/stickers when a chat is opened.
    pub auto_load_images: bool,
    /// "default" | "teal" | "slate" | "dusk" | "custom".
    pub wallpaper: String,
    /// Copy of the custom wallpaper image inside the app-data dir.
    pub wallpaper_path: Option<String>,
    /// Most-recent-first emoji, max 32.
    pub emoji_recent: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            zoom: 1.0,
            enter_to_send: true,
            send_read_receipts: true,
            auto_load_images: true,
            wallpaper: "default".into(),
            wallpaper_path: None,
            emoji_recent: Vec::new(),
        }
    }
}

impl Settings {
    fn path() -> std::path::PathBuf {
        paths::app_data_dir().join("settings.json")
    }

    pub fn load() -> Self {
        std::fs::read(Self::path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(bytes) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(Self::path(), bytes);
        }
    }

    pub fn set_zoom(&mut self, z: f32) {
        self.zoom = ((z.clamp(ZOOM_MIN, ZOOM_MAX)) * 100.0).round() / 100.0;
    }

    pub fn push_recent_emoji(&mut self, e: &str) {
        if e.chars().any(char::is_whitespace) {
            return;
        }
        self.emoji_recent.retain(|x| x != e);
        self.emoji_recent.insert(0, e.to_string());
        self.emoji_recent.truncate(EMOJI_RECENT_MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_and_recents() {
        let mut s = Settings::default();
        s.set_zoom(5.0);
        assert_eq!(s.zoom, 2.0);
        s.set_zoom(1.0 + 0.1 + 0.1);
        assert_eq!(s.zoom, 1.2);
        for e in ["a", "b", "a"] {
            s.push_recent_emoji(e);
        }
        assert_eq!(s.emoji_recent, vec!["a", "b"]);
    }
}
