//! Bridges the whatsapp-rust `Event` bus to the native UI.
//!
//! Port of the Tauri bridge: instead of serializing every event to JSON and
//! emitting it on a `wa://…` topic for a WebView to parse, each library event
//! is normalized to a typed [`UiEvent`] and pushed over an in-process channel
//! that the UI controller drains. No serialization happens on the hot path.
//!
//! Messages are still normalized to `MessageDto` here so the UI layer never
//! touches prost. History sync is decoded into per-conversation chunks.

use std::sync::Arc;

use async_channel::{Receiver, Sender};
use base64::Engine;
use whatsapp_rust::prelude::{Event, EventKind, MessageInfo};
use whatsapp_rust::wacore::proto_helpers::MessageExt;
use whatsapp_rust::wacore::types::events::{EventHandler, EventInterest};
use whatsapp_rust::waproto::whatsapp as wa;

use crate::core::dto::{
    ChatDto, ChatFlagsDto, HistoryDto, MediaDescriptorDto, MediaDto, MessageDto, MessageUpdateDto,
    QuotedDto, ReceiptDto,
};

/// Everything the UI reacts to, already normalized. Replaces the Tauri
/// `wa://…` topics one-to-one. Every variant carries its `session_id` (and
/// `ConnState` its flag) for the multi-session plumbing, even though the
/// single-account UI doesn't read them yet.
#[derive(Debug)]
#[allow(dead_code)]
// History chunks are large but rare; boxing them buys nothing on this path.
#[allow(clippy::large_enum_variant)]
pub enum UiEvent {
    /// `wa://auth/qr`
    Qr { session_id: String, code: String },
    /// `wa://auth/pair-code`
    PairCode { session_id: String, code: String },
    /// `wa://auth/paired`
    Paired { session_id: String },
    /// `wa://auth/pair-error`
    PairError { session_id: String, message: String },
    /// `wa://auth/logged-out`
    LoggedOut { session_id: String },
    /// `wa://auth/dead` — run loop ended while unpaired; QR is stale.
    ClientDead { session_id: String },
    /// `wa://conn/state` — connected or disconnected; re-query status.
    ConnState { session_id: String, connected: bool },
    /// `wa://message`
    Message { session_id: String, message: MessageDto },
    /// `wa://message/update`
    MessageUpdate { session_id: String, update: MessageUpdateDto },
    /// `wa://message/delete-for-me`
    DeleteForMe { session_id: String, chat_jid: String, message_id: String },
    /// `wa://history`
    History { session_id: String, history: HistoryDto },
    /// `wa://receipt`
    Receipt { session_id: String, receipt: ReceiptDto },
    /// `wa://chat/flags`
    ChatFlags { session_id: String, flags: ChatFlagsDto },
    /// `wa://error/stream`
    StreamError { session_id: String, message: String },
}

/// Event kinds the bridge forwards: everything the UI consumes. High-volume
/// presence/typing is never subscribed to, so the library never builds it.
const FORWARDED: &[EventKind] = &[
    EventKind::PairingQrCode,
    EventKind::PairingCode,
    EventKind::PairSuccess,
    EventKind::PairError,
    EventKind::LoggedOut,
    EventKind::Connected,
    EventKind::Disconnected,
    EventKind::Message,
    EventKind::Receipt,
    EventKind::HistorySync,
    EventKind::MuteUpdate,
    EventKind::PinUpdate,
    EventKind::ArchiveUpdate,
    EventKind::DeleteMessageForMeUpdate,
    EventKind::StreamError,
    EventKind::ConnectFailure,
    EventKind::TemporaryBan,
];

/// Library event handler feeding [`pump`], with a narrow interest so unwanted
/// events are never materialized or queued.
pub struct BridgeHandler {
    tx: Sender<Arc<Event>>,
}

impl BridgeHandler {
    pub fn new() -> (Arc<Self>, Receiver<Arc<Event>>) {
        let (tx, rx) = async_channel::unbounded();
        (Arc::new(Self { tx }), rx)
    }
}

impl EventHandler for BridgeHandler {
    fn handle_event(&self, event: Arc<Event>) {
        let _ = self.tx.try_send(event);
    }

    fn interest(&self) -> EventInterest {
        EventInterest::of(FORWARDED)
    }
}

/// Drain one session's library events into typed [`UiEvent`]s.
pub async fn pump(ui: Sender<UiEvent>, session_id: String, rx: Receiver<Arc<Event>>) {
    log::info!("[{session_id}] event bridge started");
    let sid = || session_id.clone();
    while let Ok(event) = rx.recv().await {
        let out = match event.as_ref() {
            Event::Message(msg, info) => {
                // Control messages (revoke/edit/reaction) patch an existing bubble.
                if let Some(update) = message_update_dto(msg, info) {
                    Some(UiEvent::MessageUpdate { session_id: sid(), update })
                } else if msg.get_base_message().secret_encrypted_message.is_some() {
                    // Undecryptable encrypted edit/reaction: nothing to show.
                    None
                } else {
                    let dto = message_dto_live(msg, info);
                    // Device-to-device protocol traffic / album headers: skip.
                    is_renderable(&dto).then(|| UiEvent::Message { session_id: sid(), message: dto })
                }
            }
            Event::HistorySync(lazy) => {
                spawn_history_decode(&ui, &session_id, lazy);
                None
            }
            Event::Receipt(r) => receipt_dto(r).map(|receipt| UiEvent::Receipt { session_id: sid(), receipt }),
            Event::MuteUpdate(_) | Event::PinUpdate(_) | Event::ArchiveUpdate(_) => Some(UiEvent::ChatFlags {
                session_id: sid(),
                flags: chat_flags_dto(event.as_ref()),
            }),
            Event::DeleteMessageForMeUpdate(u) => Some(UiEvent::DeleteForMe {
                session_id: sid(),
                chat_jid: u.chat_jid.to_string(),
                message_id: u.message_id.clone(),
            }),
            Event::PairingQrCode { code, .. } => Some(UiEvent::Qr { session_id: sid(), code: code.clone() }),
            Event::PairingCode { code, .. } => Some(UiEvent::PairCode { session_id: sid(), code: code.clone() }),
            Event::PairSuccess(_) => Some(UiEvent::Paired { session_id: sid() }),
            Event::PairError(e) => Some(UiEvent::PairError { session_id: sid(), message: format!("{e:?}") }),
            Event::LoggedOut(_) => Some(UiEvent::LoggedOut { session_id: sid() }),
            Event::Connected(_) => Some(UiEvent::ConnState { session_id: sid(), connected: true }),
            Event::Disconnected(_) => Some(UiEvent::ConnState { session_id: sid(), connected: false }),
            Event::StreamError(_) | Event::ConnectFailure(_) | Event::TemporaryBan(_) => Some(UiEvent::StreamError {
                session_id: sid(),
                message: format!("{:?}", event.kind()),
            }),
            _ => None,
        };
        if let Some(ev) = out
            && ui.send(ev).await.is_err()
        {
            break; // UI gone
        }
    }
    log::info!("[{session_id}] event bridge stopped");
}

/// Serializes history-sync decodes so only one conversation is alive at once.
static HISTORY_DECODE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Decode a `HistorySync` blob on a blocking thread and push one chunk per
/// conversation as it is decoded.
fn spawn_history_decode(
    ui: &Sender<UiEvent>,
    session_id: &str,
    lazy: &whatsapp_rust::wacore::types::events::LazyHistorySync,
) {
    let compressed = lazy.compressed_bytes().clone();
    let size = lazy.decompressed_size();
    let ui = ui.clone();
    let session_id = session_id.to_string();
    tokio::task::spawn_blocking(move || {
        let _guard = HISTORY_DECODE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = stream_history(&compressed, size, |chunk| {
            let _ = ui.send_blocking(UiEvent::History {
                session_id: session_id.clone(),
                history: chunk,
            });
        });
        log::info!("[{session_id}] history sync decoded: {count} conversation(s)");
    });
}

/// Walks the history blob one conversation at a time.
fn stream_history(compressed: &[u8], size: usize, mut emit: impl FnMut(HistoryDto)) -> usize {
    use whatsapp_rust::wacore::history_sync::HistorySyncStream;
    let mut stream = HistorySyncStream::new(compressed, size as u64);
    let mut count = 0;
    loop {
        match stream.next_conversation() {
            Ok(Some(conv)) => {
                emit(convert_conversation(conv));
                count += 1;
            }
            Ok(None) => break,
            Err(e) => {
                log::warn!("history decode error: {e}");
                break;
            }
        }
    }
    count
}

fn convert_conversation(conv: wa::Conversation) -> HistoryDto {
    let chat_jid = normalize_chat_jid(&conv.id);
    let mut messages: Vec<MessageDto> = conv
        .messages
        .into_iter()
        .filter_map(|hsm| hsm.message)
        .map(|wmi| message_dto_history(&chat_jid, &wmi))
        .filter(is_renderable)
        .collect();

    // Newest message drives the chat preview + timestamp fallback.
    messages.sort_by_key(|m| m.timestamp);
    let last = messages.last();
    let last_message = last.and_then(|m| m.text.clone());
    let timestamp = conv
        .last_msg_timestamp
        .map(|t| t as i64)
        .or_else(|| last.map(|m| m.timestamp))
        .unwrap_or(0);

    let chat = ChatDto {
        jid: chat_jid,
        name: conv.name,
        last_message,
        timestamp,
        unread: conv.unread_count.unwrap_or(0),
    };
    HistoryDto {
        chats: vec![chat],
        messages,
    }
}

/// Normalize a mute/pin/archive update into the unified `ChatFlagsDto`.
fn chat_flags_dto(event: &Event) -> ChatFlagsDto {
    let mut dto = ChatFlagsDto {
        jid: String::new(),
        muted: None,
        pinned: None,
        archived: None,
    };
    match event {
        Event::MuteUpdate(u) => {
            dto.jid = normalize_chat_jid(&u.jid.to_string());
            dto.muted = u.action.muted;
        }
        Event::PinUpdate(u) => {
            dto.jid = normalize_chat_jid(&u.jid.to_string());
            dto.pinned = u.action.pinned;
        }
        Event::ArchiveUpdate(u) => {
            dto.jid = normalize_chat_jid(&u.jid.to_string());
            dto.archived = u.action.archived;
        }
        _ => {}
    }
    dto
}

/// Normalize a `Receipt` to the UI shape, returning `None` for receipt types the
/// frontend doesn't render (retry/sender/server-error/etc.).
fn receipt_dto(r: &whatsapp_rust::wacore::types::events::Receipt) -> Option<ReceiptDto> {
    use whatsapp_rust::wacore::types::presence::ReceiptType as RT;
    let status = match &r.r#type {
        RT::Delivered => "delivered",
        RT::Sent => "sent",
        RT::Read | RT::ReadSelf => "read",
        RT::Played | RT::PlayedSelf => "played",
        _ => return None,
    };
    Some(ReceiptDto {
        chat_jid: normalize_chat_jid(&r.source.chat.to_string()),
        sender_jid: r.source.sender.to_string(),
        message_ids: r.message_ids.iter().map(|id| id.to_string()).collect(),
        status: status.to_string(),
        timestamp: r.timestamp.timestamp(),
    })
}

/// Detect a control message (revoke / edit / reaction) that patches an existing
/// bubble. Returns `None` for ordinary content messages.
fn message_update_dto(msg: &wa::Message, info: &MessageInfo) -> Option<MessageUpdateDto> {
    use wa::message::protocol_message::Type as PmType;
    let base = msg.get_base_message();

    let make = |kind: &str, target_id: String, text: Option<String>| MessageUpdateDto {
        chat_jid: normalize_chat_jid(&info.source.chat.to_string()),
        target_id,
        kind: kind.to_string(),
        text,
        sender_jid: Some(info.source.sender.to_string()),
        from_me: info.source.is_from_me,
        timestamp: info.timestamp.timestamp(),
    };

    if let Some(pm) = base.protocol_message.as_ref() {
        let target_id = pm.key.as_ref().and_then(|k| k.id.clone()).unwrap_or_default();
        if pm.r#type == Some(PmType::Revoke as i32) {
            return Some(make("revoke", target_id, None));
        }
        if pm.r#type == Some(PmType::MessageEdit as i32) {
            let text = pm.edited_message.as_deref().and_then(text_of);
            return Some(make("edit", target_id, text));
        }
    }

    if let Some(rm) = base.reaction_message.as_ref() {
        let target_id = rm.key.as_ref().and_then(|k| k.id.clone()).unwrap_or_default();
        // An empty/absent emoji means the reaction was removed.
        return Some(make("reaction", target_id, Some(rm.text.clone().unwrap_or_default())));
    }

    None
}

fn message_dto_live(msg: &wa::Message, info: &MessageInfo) -> MessageDto {
    let push_name = (!info.push_name.is_empty()).then(|| info.push_name.clone());
    MessageDto {
        id: info.id.clone(),
        chat_jid: normalize_chat_jid(&info.source.chat.to_string()),
        sender_jid: info.source.sender.to_string(),
        from_me: info.source.is_from_me,
        timestamp: info.timestamp.timestamp(),
        push_name,
        text: text_of(msg),
        kind: classify(msg),
        thumbnail: thumbnail_b64(msg),
        media: media_dto(msg),
        quoted: quoted_dto(msg),
        album_parent_id: album_parent_of(msg),
    }
}

/// Whether a message has anything for the UI to draw. Protocol / control
/// payloads (and history-sync system stubs, which carry no message at all)
/// classify as "other" with no text or media.
fn is_renderable(m: &MessageDto) -> bool {
    m.kind != "other" || m.text.is_some() || m.media.is_some() || m.thumbnail.is_some()
}

/// The parent message id when `msg` is an item of a media album. The album
/// association sits in `messageContextInfo`, which WA lifts to the outermost
/// message when wrapping the item (`associatedChildMessage`), so check each
/// layer on the way in.
fn album_parent_of(msg: &wa::Message) -> Option<String> {
    use wa::message_association::AssociationType;
    let mut cur = Some(msg);
    while let Some(m) = cur {
        let assoc = m
            .message_context_info
            .as_ref()
            .and_then(|c| c.message_association.as_ref());
        if let Some(a) = assoc
            && a.association_type == Some(AssociationType::MediaAlbum as i32)
            && let Some(id) = a.parent_message_key.as_ref().and_then(|k| k.id.clone())
        {
            return Some(id);
        }
        cur = m
            .device_sent_message
            .as_ref()
            .and_then(|d| d.message.as_deref())
            .or_else(|| {
                m.associated_child_message
                    .as_ref()
                    .and_then(|f| f.message.as_deref())
            });
    }
    None
}

fn message_dto_history(chat_jid: &str, wmi: &wa::WebMessageInfo) -> MessageDto {
    let key = &wmi.key;
    let from_me = key.from_me.unwrap_or(false);
    let sender_jid = wmi
        .participant
        .clone()
        .or_else(|| key.participant.clone())
        .unwrap_or_else(|| if from_me { String::new() } else { chat_jid.to_string() });

    let (text, kind, thumbnail, media, quoted, album_parent_id) = match wmi.message.as_deref() {
        Some(m) => (
            text_of(m),
            classify(m),
            thumbnail_b64(m),
            media_dto(m),
            quoted_dto(m),
            album_parent_of(m),
        ),
        None => (None, "other".to_string(), None, None, None, None),
    };

    MessageDto {
        id: key.id.clone().unwrap_or_default(),
        chat_jid: chat_jid.to_string(),
        sender_jid,
        from_me,
        timestamp: wmi.message_timestamp.unwrap_or(0) as i64,
        push_name: wmi.push_name.clone(),
        text,
        kind,
        thumbnail,
        media,
        quoted,
        album_parent_id,
    }
}

/// Canonical chat key for a JID.
///
/// Live message events carry a device/agent suffix on the chat JID
/// (`12345.0:7@s.whatsapp.net`), while history-sync conversation ids do not
/// (`12345@s.whatsapp.net`). Keying the chat map on the raw string would split
/// the same conversation in two — incoming history under one key, your own
/// phone-sent messages under another. Stripping the agent (`.N`) and device
/// (`:N`) parts unifies them on `user@server`.
///
/// Limitation: this does not bridge LID (`@lid`) and phone-number
/// (`@s.whatsapp.net`) addressing for the same contact — those have different
/// servers and would need the library's LID↔PN map to merge.
pub fn normalize_chat_jid(jid: &str) -> String {
    match jid.split_once('@') {
        Some((user, server)) => {
            let base = user.split([':', '.']).next().unwrap_or(user);
            format!("{base}@{server}")
        }
        None => jid.to_string(),
    }
}

/// Peel the FutureProof wrappers `get_base_message()` doesn't, so content nested
/// in them (group-mention links, spoilers, bot/status wrappers, lottie stickers)
/// classifies by its inner payload instead of falling through to "[other]".
fn unwrap_wrappers(msg: &wa::Message) -> &wa::Message {
    let mut cur = msg.get_base_message();
    loop {
        let next = cur
            .group_mentioned_message
            .as_ref()
            .or(cur.bot_invoke_message.as_ref())
            .or(cur.bot_forwarded_message.as_ref())
            .or(cur.status_mention_message.as_ref())
            .or(cur.group_status_mention_message.as_ref())
            .or(cur.associated_child_message.as_ref())
            .or(cur.lottie_sticker_message.as_ref())
            .or(cur.spoiler_message.as_ref())
            .and_then(|m| m.message.as_deref());
        match next {
            Some(inner) => cur = inner.get_base_message(),
            None => return cur,
        }
    }
}

fn text_of(msg: &wa::Message) -> Option<String> {
    let base = unwrap_wrappers(msg);
    base.text_content()
        .or_else(|| base.get_caption())
        .map(str::to_string)
        // Business / interactive variants (buttons, lists, templates, polls, …)
        // aren't covered by text_content/caption — extract their readable text so
        // they never render as a bare "[other]" placeholder.
        .or_else(|| structured_text(base))
}

fn classify(msg: &wa::Message) -> String {
    let b = unwrap_wrappers(msg);
    let k = if b.conversation.is_some() || b.extended_text_message.is_some() {
        "text"
    } else if b.image_message.is_some() {
        "image"
    } else if b.video_message.is_some() || b.ptv_message.is_some() {
        "video"
    } else if b.audio_message.is_some() {
        "audio"
    } else if b.document_message.is_some() {
        "document"
    } else if b.sticker_message.is_some() {
        "sticker"
    } else if b.buttons_message.is_some() || b.buttons_response_message.is_some() {
        "buttons"
    } else if b.list_message.is_some() || b.list_response_message.is_some() {
        "list"
    } else if b.interactive_message.is_some() || b.interactive_response_message.is_some() {
        "interactive"
    } else if b.template_message.is_some()
        || b.template_button_reply_message.is_some()
        || b.highly_structured_message.is_some()
    {
        "template"
    } else if b.poll_creation_message.is_some()
        || b.poll_creation_message_v2.is_some()
        || b.poll_creation_message_v3.is_some()
    {
        "poll"
    } else if b.order_message.is_some() {
        "order"
    } else if b.product_message.is_some() {
        "product"
    } else if b.contact_message.is_some() {
        "contact"
    } else if b.location_message.is_some() {
        "location"
    } else {
        "other"
    };
    k.to_string()
}

/// Best-effort human-readable text for interactive / business message variants
/// that `text_content()` doesn't cover. Choices (list rows, poll options) are
/// encoded as `•` bullet lines so the bubble can render them verbatim.
fn structured_text(b: &wa::Message) -> Option<String> {
    // Buttons: prompt (+ footer).
    if let Some(m) = b.buttons_message.as_ref() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = m.content_text.clone() {
            parts.push(t);
        }
        if let Some(t) = m.footer_text.clone() {
            parts.push(t);
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    if let Some(m) = b.buttons_response_message.as_ref() {
        match &m.response {
            Some(wa::message::buttons_response_message::Response::SelectedDisplayText(t)) => {
                return Some(t.clone());
            }
            _ => {
                if let Some(t) = m.selected_button_id.clone() {
                    return Some(t);
                }
            }
        }
    }
    // List: title / description + row choices.
    if let Some(m) = b.list_message.as_ref() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = m.title.clone() {
            parts.push(t);
        }
        if let Some(t) = m.description.clone() {
            parts.push(t);
        }
        for row in m.sections.iter().flat_map(|s| s.rows.iter()) {
            if let Some(t) = row.title.clone() {
                parts.push(format!("• {t}"));
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    if let Some(m) = b.list_response_message.as_ref() {
        if let Some(t) = m.title.clone() {
            return Some(t);
        }
    }
    // Interactive (native flow / carousel): header title + body + footer.
    if let Some(m) = b.interactive_message.as_ref() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = m.header.as_deref().and_then(|h| h.title.clone()) {
            parts.push(t);
        }
        if let Some(t) = m.body.as_ref().and_then(|x| x.text.clone()) {
            parts.push(t);
        }
        if let Some(t) = m.footer.as_deref().and_then(|f| f.text.clone()) {
            parts.push(t);
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    if let Some(m) = b.interactive_response_message.as_ref() {
        if let Some(t) = m.body.as_ref().and_then(|x| x.text.clone()) {
            return Some(t);
        }
    }
    // Template (pre-approved business): hydrated title + content + footer.
    if let Some(ht) = b
        .template_message
        .as_ref()
        .and_then(|m| m.hydrated_template.as_deref())
    {
        let mut parts: Vec<String> = Vec::new();
        if let Some(wa::message::template_message::hydrated_four_row_template::Title::HydratedTitleText(t)) =
            &ht.title
        {
            parts.push(t.clone());
        }
        if let Some(t) = ht.hydrated_content_text.clone() {
            parts.push(t);
        }
        if let Some(t) = ht.hydrated_footer_text.clone() {
            parts.push(t);
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    if let Some(m) = b.template_button_reply_message.as_ref() {
        if let Some(t) = m.selected_display_text.clone() {
            return Some(t);
        }
    }
    if let Some(m) = b.highly_structured_message.as_ref() {
        if let Some(t) = m
            .hydrated_hsm
            .as_deref()
            .and_then(|t| t.hydrated_template.as_deref())
            .and_then(|ht| ht.hydrated_content_text.clone())
        {
            return Some(t);
        }
        if let Some(t) = m.element_name.clone() {
            return Some(t);
        }
    }
    // Poll: question + options (any protocol version).
    let poll = b
        .poll_creation_message
        .as_deref()
        .or(b.poll_creation_message_v2.as_deref())
        .or(b.poll_creation_message_v3.as_deref());
    if let Some(p) = poll {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = p.name.clone() {
            parts.push(t);
        }
        for o in p.options.iter() {
            if let Some(t) = o.option_name.clone() {
                parts.push(format!("• {t}"));
            }
        }
        if !parts.is_empty() {
            return Some(parts.join("\n"));
        }
    }
    // Order.
    if let Some(m) = b.order_message.as_ref() {
        if let Some(t) = m.order_title.clone().or_else(|| m.message.clone()) {
            return Some(t);
        }
    }
    // Product catalog item.
    if let Some(m) = b.product_message.as_ref() {
        if let Some(t) = m
            .product
            .as_deref()
            .and_then(|p| p.title.clone())
            .or_else(|| m.body.clone())
        {
            return Some(t);
        }
    }
    // Shared contact.
    if let Some(m) = b.contact_message.as_ref() {
        if let Some(t) = m.display_name.clone() {
            return Some(t);
        }
    }
    // Location.
    if let Some(m) = b.location_message.as_ref() {
        return Some(m.name.clone().or_else(|| m.address.clone()).unwrap_or_else(|| {
            format!(
                "{:.5}, {:.5}",
                m.degrees_latitude.unwrap_or(0.0),
                m.degrees_longitude.unwrap_or(0.0)
            )
        }));
    }
    None
}

/// Build a downloadable media descriptor + display info from a message's media
/// payload. Returns `None` for non-media or media missing the keys needed to
/// decrypt (in which case the inline thumbnail still renders).
fn media_dto(msg: &wa::Message) -> Option<MediaDto> {
    let b = unwrap_wrappers(msg);
    let b64 = |bytes: &[u8]| base64::engine::general_purpose::STANDARD.encode(bytes);

    let descriptor = |direct_path: &Option<String>,
                      media_key: &Option<Vec<u8>>,
                      file_sha256: &Option<Vec<u8>>,
                      file_enc_sha256: &Option<Vec<u8>>,
                      file_length: Option<u64>,
                      media_type: &str|
     -> Option<MediaDescriptorDto> {
        Some(MediaDescriptorDto {
            direct_path: direct_path.clone()?,
            media_key: b64(media_key.as_ref()?),
            file_sha256: b64(file_sha256.as_ref()?),
            file_enc_sha256: b64(file_enc_sha256.as_ref()?),
            file_length: file_length.unwrap_or(0),
            media_type: media_type.to_string(),
        })
    };

    if let Some(m) = b.image_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "image")?;
        return Some(MediaDto {
            kind: "image".into(),
            mimetype: m.mimetype.clone(),
            file_name: None,
            width: m.width,
            height: m.height,
            duration_secs: None,
            is_animated: None,
            descriptor: d,
        });
    }
    if let Some(m) = b.video_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "video")?;
        return Some(MediaDto {
            kind: "video".into(),
            mimetype: m.mimetype.clone(),
            file_name: None,
            width: m.width,
            height: m.height,
            duration_secs: m.seconds,
            is_animated: m.gif_playback,
            descriptor: d,
        });
    }
    if let Some(m) = b.ptv_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "video")?;
        return Some(MediaDto {
            kind: "video".into(),
            mimetype: m.mimetype.clone(),
            file_name: None,
            width: m.width,
            height: m.height,
            duration_secs: m.seconds,
            is_animated: m.gif_playback,
            descriptor: d,
        });
    }
    if let Some(m) = b.audio_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "audio")?;
        return Some(MediaDto {
            kind: "audio".into(),
            mimetype: m.mimetype.clone(),
            file_name: None,
            width: None,
            height: None,
            duration_secs: m.seconds,
            is_animated: None,
            descriptor: d,
        });
    }
    if let Some(m) = b.document_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "document")?;
        return Some(MediaDto {
            kind: "document".into(),
            mimetype: m.mimetype.clone(),
            file_name: m.file_name.clone(),
            width: None,
            height: None,
            duration_secs: None,
            is_animated: None,
            descriptor: d,
        });
    }
    if let Some(m) = b.sticker_message.as_ref() {
        let d = descriptor(&m.direct_path, &m.media_key, &m.file_sha256, &m.file_enc_sha256, m.file_length, "sticker")?;
        return Some(MediaDto {
            kind: "sticker".into(),
            mimetype: m.mimetype.clone(),
            file_name: None,
            width: m.width,
            height: m.height,
            duration_secs: None,
            is_animated: m.is_animated,
            descriptor: d,
        });
    }
    None
}

/// Find the `ContextInfo` carried by whichever message variant has one.
fn context_info_of(base: &wa::Message) -> Option<&wa::ContextInfo> {
    macro_rules! first_ci {
        ($($f:ident),+ $(,)?) => {
            $(
                if let Some(c) = base.$f.as_ref().and_then(|m| m.context_info.as_deref()) {
                    return Some(c);
                }
            )+
        };
    }
    first_ci!(
        extended_text_message,
        image_message,
        video_message,
        audio_message,
        document_message,
        sticker_message,
    );
    None
}

/// Extract the quoted message for a reply (`ContextInfo` with a `stanza_id`).
fn quoted_dto(msg: &wa::Message) -> Option<QuotedDto> {
    let ci = context_info_of(msg.get_base_message())?;
    let id = ci.stanza_id.clone()?; // a real reply always references a stanza
    let qm = ci.quoted_message.as_deref();
    let (text, kind, thumbnail) = match qm {
        Some(m) => (text_of(m), classify(m), thumbnail_b64(m)),
        None => (None, "other".to_string(), None),
    };
    Some(QuotedDto {
        id,
        sender_jid: ci.participant.clone(),
        text,
        kind,
        thumbnail,
    })
}

fn thumbnail_b64(msg: &wa::Message) -> Option<String> {
    let b = unwrap_wrappers(msg);
    let bytes = b
        .image_message
        .as_ref()
        .and_then(|m| m.jpeg_thumbnail.as_ref())
        .or_else(|| b.video_message.as_ref().and_then(|m| m.jpeg_thumbnail.as_ref()))
        .or_else(|| b.ptv_message.as_ref().and_then(|m| m.jpeg_thumbnail.as_ref()))
        .or_else(|| b.extended_text_message.as_ref().and_then(|m| m.jpeg_thumbnail.as_ref()))?;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

