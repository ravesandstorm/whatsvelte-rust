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

use crate::dto::{
    ChatDto, ChatFlagsDto, HistoryDto, MediaDescriptorDto, MediaDto, MessageDto, MessageUpdateDto,
    QuotedDto, ReceiptDto, SyncProgressDto,
};

/// Catch-all topic every event (except the noisy/streamed ones) is also emitted on.
const GLOBAL_TOPIC: &str = "wa://event";

pub async fn pump(app: AppHandle, session_id: String, rx: Receiver<Arc<Event>>) {
    log::info!("[{session_id}] event bridge started");
    while let Ok(event) = rx.recv().await {
        let kind = event.kind();

        let payload: Value = match event.as_ref() {
            Event::Message(msg, info) => {
                // Control messages (revoke/edit/reaction) patch an existing
                // bubble rather than adding one — route them to a separate topic.
                if let Some(update) = message_update_dto(msg, info) {
                    let envelope = json!({
                        "sessionId": session_id,
                        "kind": "MessageUpdate",
                        "payload": serde_json::to_value(update).unwrap_or(Value::Null),
                    });
                    let _ = app.emit("wa://message/update", envelope.clone());
                    let _ = app.emit(GLOBAL_TOPIC, envelope);
                    continue;
                }
                serde_json::to_value(message_dto_live(msg, info)).unwrap_or(Value::Null)
            }
            // History sync is large and lazy; decode it off-thread and emit
            // per-conversation chunks on wa://history, then move on.
            Event::HistorySync(lazy) => {
                spawn_history_decode(&app, &session_id, lazy);
                continue;
            }
            // Normalize delivery/read/played receipts; skip the protocol-internal
            // ones (retry, sender, server-error, …) the UI has no use for.
            Event::Receipt(r) => match receipt_dto(r) {
                Some(dto) => serde_json::to_value(dto).unwrap_or(Value::Null),
                None => continue,
            },
            // Server-synced chat flags → one unified shape on wa://chat/flags.
            Event::MuteUpdate(_) | Event::PinUpdate(_) | Event::ArchiveUpdate(_) => {
                let dto = chat_flags_dto(event.as_ref());
                let envelope = json!({
                    "sessionId": session_id,
                    "kind": "ChatFlags",
                    "payload": serde_json::to_value(dto).unwrap_or(Value::Null),
                });
                let _ = app.emit("wa://chat/flags", envelope.clone());
                let _ = app.emit(GLOBAL_TOPIC, envelope);
                continue;
            }
            // Offline-sync progress → a single shape on wa://sync/progress so the
            // UI can drive a "loading your chats…" bar after the handshake.
            Event::OfflineSyncPreview(p) => {
                let dto = SyncProgressDto {
                    phase: "preview".into(),
                    total: p.total,
                    messages: p.messages,
                    notifications: p.notifications,
                    receipts: p.receipts,
                    app_data_changes: p.app_data_changes,
                    done: false,
                };
                let envelope = json!({
                    "sessionId": session_id,
                    "kind": "OfflineSyncPreview",
                    "payload": serde_json::to_value(dto).unwrap_or(Value::Null),
                });
                let _ = app.emit("wa://sync/progress", envelope.clone());
                let _ = app.emit(GLOBAL_TOPIC, envelope);
                continue;
            }
            Event::OfflineSyncCompleted(c) => {
                let dto = SyncProgressDto {
                    phase: "completed".into(),
                    total: c.count,
                    messages: 0,
                    notifications: 0,
                    receipts: 0,
                    app_data_changes: 0,
                    done: true,
                };
                let envelope = json!({
                    "sessionId": session_id,
                    "kind": "OfflineSyncCompleted",
                    "payload": serde_json::to_value(dto).unwrap_or(Value::Null),
                });
                let _ = app.emit("wa://sync/progress", envelope.clone());
                let _ = app.emit(GLOBAL_TOPIC, envelope);
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

    let (text, kind, thumbnail, media, quoted) = match wmi.message.as_deref() {
        Some(m) => (
            text_of(m),
            classify(m),
            thumbnail_b64(m),
            media_dto(m),
            quoted_dto(m),
        ),
        None => (None, "other".to_string(), None, None, None),
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
        // Business / interactive variants (buttons, lists, templates, polls, …)
        // aren't covered by text_content/caption — extract their readable text so
        // they never render as a bare "[other]" placeholder.
        .or_else(|| structured_text(msg.get_base_message()))
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
    let b = msg.get_base_message();
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
    let (text, kind) = match qm {
        Some(m) => (text_of(m), classify(m)),
        None => (None, "other".to_string()),
    };
    Some(QuotedDto {
        id,
        sender_jid: ci.participant.clone(),
        text,
        kind,
    })
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
