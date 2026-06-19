//! Bridges the whatsapp-rust `Event` bus to Tauri events.
//!
//! Each library `Event` is re-emitted on a per-kind topic (`wa://message`,
//! `wa://auth/qr`, …) and on the catch-all `wa://event`. Every payload is
//! wrapped with `sessionId` so a multi-session frontend knows the account.
//!
//! Messages are normalized to `MessageDto` here so the frontend never parses
//! prost. History sync is decoded into per-conversation `wa://history` chunks.

use std::sync::Arc;

use async_channel::Receiver;
use base64::Engine;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use whatsapp_rust::prelude::{Event, EventKind, MessageInfo};
use whatsapp_rust::wacore::proto_helpers::MessageExt;
use whatsapp_rust::waproto::whatsapp as wa;

use crate::dto::{ChatDto, HistoryDto, MessageDto};

/// Catch-all topic every event (except the noisy/streamed ones) is also emitted on.
const GLOBAL_TOPIC: &str = "wa://event";

pub async fn pump(app: AppHandle, session_id: String, rx: Receiver<Arc<Event>>) {
    log::info!("[{session_id}] event bridge started");
    while let Ok(event) = rx.recv().await {
        let kind = event.kind();

        let payload: Value = match event.as_ref() {
            Event::Message(msg, info) => {
                serde_json::to_value(message_dto_live(msg, info)).unwrap_or(Value::Null)
            }
            // History sync is large and lazy; decode it off-thread and emit
            // per-conversation chunks on wa://history, then move on.
            Event::HistorySync(lazy) => {
                spawn_history_decode(&app, &session_id, lazy);
                continue;
            }
            // Flatten pairing events to the documented {code, timeoutSecs} shape
            // (otherwise they serialize externally-tagged and the UI can't read
            // `payload.code`).
            Event::PairingQrCode { code, timeout } => {
                json!({ "code": code, "timeoutSecs": timeout.as_secs() })
            }
            Event::PairingCode { code, timeout } => {
                json!({ "code": code, "timeoutSecs": timeout.as_secs() })
            }
            // Not serializable / too noisy for the UI; skip entirely.
            Event::Notification(_) | Event::RawNode(_) => continue,
            // Everything else: externally-tagged JSON ({"Connected":null}, …).
            // The `kind` field disambiguates.
            other => serde_json::to_value(other).unwrap_or(Value::Null),
        };

        let envelope = json!({
            "sessionId": session_id,
            "kind": format!("{kind:?}"),
            "payload": payload,
        });

        let topic = topic_for(kind);
        if topic != GLOBAL_TOPIC {
            let _ = app.emit(topic, envelope.clone());
        }
        let _ = app.emit(GLOBAL_TOPIC, envelope);
    }
    log::info!("[{session_id}] event bridge stopped");
}

/// Decodes a `HistorySync` blob on a blocking thread (zlib + protobuf), then
/// emits one `wa://history` chunk per conversation so the UI renders
/// progressively without the whole sync ever being materialized at once.
fn spawn_history_decode(
    app: &AppHandle,
    session_id: &str,
    lazy: &whatsapp_rust::wacore::types::events::LazyHistorySync,
) {
    let compressed = lazy.compressed_bytes().clone();
    let size = lazy.decompressed_size();
    let app = app.clone();
    let session_id = session_id.to_string();
    tauri::async_runtime::spawn(async move {
        let chunks = tauri::async_runtime::spawn_blocking(move || decode_history(&compressed, size))
            .await
            .unwrap_or_default();
        log::info!("[{session_id}] history sync decoded: {} conversation(s)", chunks.len());
        for chunk in chunks {
            let envelope = json!({
                "sessionId": session_id,
                "kind": "HistorySync",
                "payload": chunk,
            });
            let _ = app.emit("wa://history", envelope);
        }
    });
}

fn decode_history(compressed: &[u8], size: usize) -> Vec<HistoryDto> {
    use whatsapp_rust::wacore::history_sync::HistorySyncStream;
    let mut stream = HistorySyncStream::new(compressed, size as u64);
    let mut out = Vec::new();
    loop {
        match stream.next_conversation() {
            Ok(Some(conv)) => out.push(convert_conversation(conv)),
            Ok(None) => break,
            Err(e) => {
                log::warn!("history decode error: {e}");
                break;
            }
        }
    }
    out
}

fn convert_conversation(conv: wa::Conversation) -> HistoryDto {
    let chat_jid = normalize_chat_jid(&conv.id);
    let mut messages: Vec<MessageDto> = conv
        .messages
        .into_iter()
        .filter_map(|hsm| hsm.message)
        .map(|wmi| message_dto_history(&chat_jid, &wmi))
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
    }
}

fn message_dto_history(chat_jid: &str, wmi: &wa::WebMessageInfo) -> MessageDto {
    let key = &wmi.key;
    let from_me = key.from_me.unwrap_or(false);
    let sender_jid = wmi
        .participant
        .clone()
        .or_else(|| key.participant.clone())
        .unwrap_or_else(|| if from_me { String::new() } else { chat_jid.to_string() });

    let (text, kind, thumbnail) = match wmi.message.as_deref() {
        Some(m) => (text_of(m), classify(m), thumbnail_b64(m)),
        None => (None, "other".to_string(), None),
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
fn normalize_chat_jid(jid: &str) -> String {
    match jid.split_once('@') {
        Some((user, server)) => {
            let base = user.split([':', '.']).next().unwrap_or(user);
            format!("{base}@{server}")
        }
        None => jid.to_string(),
    }
}

fn text_of(msg: &wa::Message) -> Option<String> {
    msg.text_content()
        .or_else(|| msg.get_caption())
        .map(str::to_string)
}

fn classify(msg: &wa::Message) -> String {
    let b = msg.get_base_message();
    let k = if b.conversation.is_some() || b.extended_text_message.is_some() {
        "text"
    } else if b.image_message.is_some() {
        "image"
    } else if b.video_message.is_some() {
        "video"
    } else if b.audio_message.is_some() {
        "audio"
    } else if b.document_message.is_some() {
        "document"
    } else if b.sticker_message.is_some() {
        "sticker"
    } else {
        "other"
    };
    k.to_string()
}

fn thumbnail_b64(msg: &wa::Message) -> Option<String> {
    let b = msg.get_base_message();
    let bytes = b
        .image_message
        .as_ref()
        .and_then(|m| m.jpeg_thumbnail.as_ref())
        .or_else(|| b.video_message.as_ref().and_then(|m| m.jpeg_thumbnail.as_ref()))?;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// Maps an event kind to its dedicated Tauri topic. Kinds without a dedicated
/// topic fall through to the catch-all only.
fn topic_for(kind: EventKind) -> &'static str {
    match kind {
        EventKind::PairingQrCode => "wa://auth/qr",
        EventKind::PairingCode => "wa://auth/pair-code",
        EventKind::PairSuccess => "wa://auth/paired",
        EventKind::PairError => "wa://auth/pair-error",
        EventKind::LoggedOut => "wa://auth/logged-out",
        EventKind::Connected | EventKind::Disconnected => "wa://conn/state",
        EventKind::Message => "wa://message",
        EventKind::Receipt => "wa://receipt",
        EventKind::Presence | EventKind::ChatPresence => "wa://presence",
        EventKind::GroupUpdate => "wa://group/update",
        EventKind::ContactUpdated | EventKind::PushNameUpdate => "wa://contact/update",
        EventKind::IncomingCall => "wa://call",
        EventKind::StreamError | EventKind::ConnectFailure | EventKind::TemporaryBan => {
            "wa://error/stream"
        }
        _ => GLOBAL_TOPIC,
    }
}
