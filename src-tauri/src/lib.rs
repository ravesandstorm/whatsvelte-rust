//! Whatsvelte-Rust — Tauri backend (the "monolith").
//!
//! Owns long-lived whatsapp-rust `Client`s (one per session), exposes their
//! operations as Tauri commands, and forwards the library event stream to the
//! UI as Tauri events. See `docs/ARCHITECTURE.md`.

mod bridge;
mod commands;
mod dto;
mod error;
#[cfg(debug_assertions)]
mod mock;
mod session;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;

use session::{SessionManager, DEFAULT_SESSION};

/// Where per-session SQLite files live.
///
/// Dev: the vendored `rust-backend/` dir (resolved at build time, so it's
/// independent of the runtime working directory). Override with `WA_DATA_DIR`.
/// Phase 4 will switch the default to the OS app-data dir.
// `app` is only used in release builds (the dev branch is a compile-time path),
// so it reads as unused under debug_assertions.
#[cfg_attr(debug_assertions, allow(unused_variables))]
fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(dir) = std::env::var("WA_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(debug_assertions)]
    {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../rust-backend"))
    }
    #[cfg(not(debug_assertions))]
    {
        let path = app.path().app_data_dir().expect("Failed to get app_data_dir");
        let _ = std::fs::create_dir_all(&path);
        path
    }
}

/// Debug builds with `WA_MOCK=1`: the same app and webview, on fake data. See `mock.rs`.
#[cfg(debug_assertions)]
fn run_mock(context: tauri::Context) {
    log::warn!("WA_MOCK is set: running on the mock backend, with no WhatsApp connection");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            mock::start_control_channel(app.handle().clone());
            Ok(())
        })
        .invoke_handler(mock::handler())
        .run(context)
        .expect("error while running tauri application");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Only one per binary: it embeds the Info.plist symbol on macOS.
    let context = tauri::generate_context!();

    #[cfg(debug_assertions)]
    if std::env::var_os("WA_MOCK").is_some() {
        return run_mock(context);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let manager = Arc::new(SessionManager::new(app.handle().clone(), data_dir(app.handle())));
            app.manage(manager.clone());

            // Auto-boot the default session so a QR code appears on launch
            // without the frontend having to ask first.
            tauri::async_runtime::spawn(async move {
                if let Err(e) = manager.get_or_create(DEFAULT_SESSION).await {
                    log::error!("failed to boot default session: {e:?}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_status,
            commands::auth_start_qr,
            commands::auth_start_pair_code,
            commands::connect,
            commands::disconnect,
            commands::auth_logout,
            commands::reset_session,
            commands::send_text,
            commands::send_reply,
            commands::edit_message,
            commands::revoke_message,
            commands::delete_for_me,
            commands::send_reaction,
            commands::mark_read,
            commands::mark_read_messages,
            commands::get_contact,
            commands::get_profile_picture_url,
            commands::resolve_jid,
            commands::download_media,
            commands::save_media_to_downloads,
            commands::send_media,
            commands::set_chat_muted,
            commands::set_chat_pinned,
            commands::set_chat_archived,
            commands::list_newsletters,
        ])
        .run(context)
        .expect("error while running tauri application");
}
