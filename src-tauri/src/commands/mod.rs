//! Tauri command handlers (the IPC surface). Grouped by area; each is a thin
//! wrapper over the whatsapp-rust `Client`. All accept an optional `sessionId`
//! (defaults to "default") and return `Result<_, ApiError>`.

mod auth;
mod chat_settings;
mod contacts;
mod media;
mod messaging;
mod newsletters;

pub use auth::*;
pub use chat_settings::*;
pub use contacts::*;
pub use media::*;
pub use messaging::*;
pub use newsletters::*;
