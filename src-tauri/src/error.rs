//! Command error type. Serializes to `{ code, message }` so the JS side can
//! `switch` on `code` instead of string-matching messages.

use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "code", content = "message")]
// NotLoggedIn / NotConnected are part of the documented error surface; the
// Phase 1 command set doesn't raise them yet.
#[allow(dead_code)]
pub enum ApiError {
    #[error("not logged in")]
    NotLoggedIn,
    #[error("not connected")]
    NotConnected,
    #[error("invalid jid: {0}")]
    InvalidJid(String),
    /// Anything bubbling up from the whatsapp-rust library, flattened to a
    /// string. Phase 2 can split the common cases into typed variants.
    #[error("{0}")]
    Library(String),
}

impl ApiError {
    pub fn library(e: impl std::fmt::Display) -> Self {
        ApiError::Library(e.to_string())
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
