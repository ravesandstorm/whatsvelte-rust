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

    /// Like `library`, but walks the `source()` chain so wrapped causes aren't
    /// hidden behind a generic top-level message — e.g. `PairError`'s
    /// "pair-code IQ request failed" becomes "...: client is not connected".
    pub fn source_chain(e: impl std::error::Error) -> Self {
        use std::fmt::Write;
        let mut msg = e.to_string();
        let mut src = e.source();
        while let Some(s) = src {
            let _ = write!(msg, ": {s}");
            src = s.source();
        }
        ApiError::Library(msg)
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
