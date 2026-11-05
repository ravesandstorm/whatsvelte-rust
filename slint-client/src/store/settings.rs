//! User preferences (port of `settings.svelte.ts` + theme/zoom stores).
//! Stored as `<app-data>/settings.json`, independent of the linked account.

use serde::{Deserialize, Serialize};

use crate::core::paths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "system" | "light" | "dark".
    pub theme: String,
    /// UI zoom factor (font + spacing scale), 0.8 ..= 1.5.
    pub zoom: f32,
    /// Send read receipts when a chat is opened.
    pub send_read_receipts: bool,
    /// Auto-download images/stickers when a chat is opened.
    pub auto_load_images: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            zoom: 1.0,
            send_read_receipts: true,
            auto_load_images: true,
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
}
