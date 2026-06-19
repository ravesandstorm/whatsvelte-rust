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

/// One rendered message. Built in `bridge.rs` from either a live
/// `Event::Message` or a history-sync `WebMessageInfo`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
    pub id: String,
    pub chat_jid: String,
    pub sender_jid: String,
    pub from_me: bool,
    /// Unix epoch seconds.
    pub timestamp: i64,
    pub push_name: Option<String>,
    pub text: Option<String>,
    /// "text" | "image" | "video" | "audio" | "document" | "sticker" | "other".
    pub kind: String,
    /// base64 JPEG (image/video thumbnail), when present inline in the proto.
    pub thumbnail: Option<String>,
}

/// One chat row for the sidebar.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDto {
    pub jid: String,
    pub name: Option<String>,
    pub last_message: Option<String>,
    /// Unix epoch seconds.
    pub timestamp: i64,
    pub unread: u32,
}

/// A history-sync chunk (emitted per conversation on `wa://history`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryDto {
    pub chats: Vec<ChatDto>,
    pub messages: Vec<MessageDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDto {
    pub jid: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}
