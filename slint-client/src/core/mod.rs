//! Tauri-free backend glue: owns the whatsapp-rust clients and exposes their
//! operations to the native UI. Ported from `src-tauri/src` with the IPC layer
//! (commands, events, asset protocol) replaced by direct calls and channels.

pub mod api;
pub mod bridge;
pub mod dto;
pub mod error;
pub mod media;
pub mod paths;
pub mod session;
