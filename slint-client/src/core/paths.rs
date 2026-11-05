//! Filesystem locations (replaces Tauri's `app.path()` resolver).

use std::path::PathBuf;

/// Same bundle identifier the Tauri build used, so an existing install's
/// session database and media cache are picked up after the switch.
pub const APP_IDENTIFIER: &str = "com.ravesandstorm.whatsvelte-rust";

/// Where per-session SQLite files live.
///
/// `WA_DATA_DIR` overrides. Debug builds use the vendored `rust-backend/` dir
/// (as the Tauri dev loop did); release builds use the OS app-data dir.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WA_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if cfg!(debug_assertions) {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../rust-backend"))
    } else {
        app_data_dir()
    }
}

/// OS app-data dir for this app (`~/Library/Application Support/<id>` etc.).
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(APP_IDENTIFIER);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Content-addressed decrypted-media cache (same location as the Tauri build).
pub fn media_dir() -> PathBuf {
    let dir = app_data_dir().join("media");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// UI-side cache (chat list, message history, learned names) — replaces the
/// WebView's IndexedDB/localStorage.
pub fn ui_cache_dir() -> PathBuf {
    let dir = app_data_dir().join("ui-cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// The user's Downloads folder.
pub fn download_dir() -> Option<PathBuf> {
    dirs::download_dir()
}
