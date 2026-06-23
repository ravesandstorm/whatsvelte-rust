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
    /// Persistent "already paired" flag (survives restarts even before the
    /// connection's <success> sets logged_in). Lets the UI show a loading screen
    /// instead of the QR on relaunch.
    pub registered: bool,
    /// Our own profile name ("username"), once synced.
    pub push_name: Option<String>,
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

/// Everything needed to fetch + decrypt one media payload. All hashes/keys are
/// base64 (raw bytes don't survive JSON). Mirrors `DownloadParams::encrypted`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDescriptorDto {
    pub direct_path: String,
    pub media_key: String,
    pub file_sha256: String,
    pub file_enc_sha256: String,
    pub file_length: u64,
    /// "image" | "video" | "audio" | "document" | "sticker".
    pub media_type: String,
}

/// Display + download info for a media message.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    pub kind: String,
    pub mimetype: Option<String>,
    pub file_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<u32>,
    pub is_animated: Option<bool>,
    pub descriptor: MediaDescriptorDto,
}

/// The message a reply quotes (from `ContextInfo`). Just enough to render the
/// quoted preview inside the replying bubble.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotedDto {
    /// Stanza id of the quoted message (links to the original).
    pub id: String,
    pub sender_jid: Option<String>,
    pub text: Option<String>,
    /// "text" | "image" | ... — for a non-text preview label.
    pub kind: String,
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
    /// Download descriptor + display info for media messages (None for text).
    pub media: Option<MediaDto>,
    /// The quoted message, when this is a reply.
    pub quoted: Option<QuotedDto>,
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

/// A receipt acknowledging one or more of our sent messages (or others'
/// messages in a chat). Emitted on `wa://receipt`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptDto {
    pub chat_jid: String,
    pub sender_jid: String,
    pub message_ids: Vec<String>,
    /// "delivered" | "read" | "played" | "sent" — normalized from `ReceiptType`.
    pub status: String,
    /// Unix epoch seconds.
    pub timestamp: i64,
}

/// A server-synced chat flag change (mute / pin / archive). Each field is set
/// only when that event carried it. Emitted on `wa://chat/flags`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatFlagsDto {
    pub jid: String,
    pub muted: Option<bool>,
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
}

/// A patch to an already-rendered message: a revoke (delete), an edit, or a
/// reaction. Emitted on `wa://message/update`. References the target by id.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdateDto {
    pub chat_jid: String,
    pub target_id: String,
    /// "revoke" | "edit" | "reaction".
    pub kind: String,
    /// New text for an edit; the emoji for a reaction ("" clears it).
    pub text: Option<String>,
    /// Who performed the update (reaction author / editor).
    pub sender_jid: Option<String>,
    pub from_me: bool,
    /// Unix epoch seconds.
    pub timestamp: i64,
}

/// Resolved LID ↔ phone-number identities for a JID. Either field may be `None`
/// if no mapping is known yet.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveJidDto {
    /// Phone-number JID, e.g. "15551234567@s.whatsapp.net".
    pub pn: Option<String>,
    /// LID JID, e.g. "100000012345678@lid".
    pub lid: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDto {
    pub jid: String,
    /// Best display name we can resolve server-side (verified business name).
    /// Regular saved-contact names are not server-queryable in this library.
    pub name: Option<String>,
    /// Verified business name, when the contact is a business account.
    pub verified_name: Option<String>,
    /// The contact's `@lid` identity, when known (used to unify LID/PN — M10).
    pub lid: Option<String>,
    pub picture_url: Option<String>,
}
