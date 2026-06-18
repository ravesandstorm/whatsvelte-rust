//! Data shapes that cross the IPC boundary. Kept deliberately small and
//! JS-friendly (camelCase, JIDs as strings) so the frontend never sees prost or
//! wacore internals.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusDto {
    pub session_id: String,
    pub logged_in: bool,
    pub connected: bool,
    /// Our own phone-number JID once paired, e.g. "15551234567@s.whatsapp.net".
    pub jid: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairCodeDto {
    pub session_id: String,
    /// 8-char code the user types into WhatsApp > Linked Devices.
    pub code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResultDto {
    pub session_id: String,
    pub message_id: String,
    pub to: String,
}
