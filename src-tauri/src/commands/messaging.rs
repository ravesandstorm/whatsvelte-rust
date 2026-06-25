//! Messaging commands.

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::wacore::proto_helpers::{build_quote_context, build_reaction_message, MessageExt};
use whatsapp_rust::waproto::whatsapp as wa;
use whatsapp_rust::{Jid, RevokeType};

use crate::dto::SendResultDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

/// Send a plain text message to a JID (user or group).
#[tauri::command]
pub async fn send_text(
    jid: String,
    text: String,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<SendResultDto> {
    let (id, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;
    let result = session
        .client
        .send_text(to, text)
        .await
        .map_err(ApiError::library)?;
    Ok(SendResultDto {
        session_id: id,
        message_id: result.message_id,
        to: result.to.to_string(),
    })
}

/// Send a text reply that quotes an existing message. The quoted preview is a
/// best-effort text snapshot (the backend keeps no message store to reconstruct
/// the original proto); the `quotedId` is what actually links the reply.
#[tauri::command]
pub async fn send_reply(
    jid: String,
    text: String,
    quoted_id: String,
    quoted_sender: String,
    quoted_text: Option<String>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<SendResultDto> {
    let (id, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;

    let mut msg = wa::Message {
        extended_text_message: Some(Box::new(wa::message::ExtendedTextMessage {
            text: Some(text),
            ..Default::default()
        })),
        ..Default::default()
    };
    let quoted = wa::Message {
        conversation: quoted_text.filter(|s| !s.is_empty()),
        ..Default::default()
    };
    msg.set_context_info(build_quote_context(quoted_id, quoted_sender, &quoted));

    let result = session
        .client
        .send_message(to, msg)
        .await
        .map_err(ApiError::library)?;
    Ok(SendResultDto {
        session_id: id,
        message_id: result.message_id,
        to: result.to.to_string(),
    })
}

/// Edit one of our own text messages in place (legacy protocolMessage path).
#[tauri::command]
pub async fn edit_message(
    jid: String,
    original_id: String,
    new_text: String,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<SendResultDto> {
    let (id, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;
    let new_content = wa::Message {
        conversation: Some(new_text),
        ..Default::default()
    };
    let message_id = session
        .client
        .edit_message(to.clone(), original_id, new_content)
        .await
        .map_err(ApiError::library)?;
    Ok(SendResultDto {
        session_id: id,
        message_id,
        to: to.to_string(),
    })
}

/// React to a message with an emoji (empty string removes our reaction). For a
/// group message, pass the target's sender as `participant`.
#[tauri::command]
pub async fn send_reaction(
    jid: String,
    target_id: String,
    from_me: bool,
    emoji: String,
    participant: Option<String>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let key = wa::MessageKey {
        remote_jid: Some(jid),
        from_me: Some(from_me),
        id: Some(target_id),
        participant,
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let msg = build_reaction_message(key, emoji, now_ms);
    session
        .client
        .send_message(chat, msg)
        .await
        .map_err(ApiError::library)?;
    Ok(())
}

/// Delete a message for everyone (revoke). For your own message pass
/// `participant: None`; as a group admin deleting another user's message, pass
/// that user's JID (as-is, no LID→PN canonicalization) as `participant`.
#[tauri::command]
pub async fn revoke_message(
    jid: String,
    message_id: String,
    participant: Option<String>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let to = parse_jid(&jid)?;
    let revoke_type = match participant {
        Some(p) => RevokeType::Admin {
            original_sender: parse_jid(&p)?,
        },
        None => RevokeType::Sender,
    };
    session
        .client
        .revoke_message(to, message_id, revoke_type)
        .await
        .map_err(ApiError::library)?;
    Ok(())
}

/// Delete a message locally only ("delete for me"). For a group message from
/// someone else, pass their JID as `participant`.
#[tauri::command]
pub async fn delete_for_me(
    jid: String,
    message_id: String,
    from_me: bool,
    participant: Option<String>,
    timestamp: Option<i64>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let participant_jid = match participant {
        Some(p) => Some(parse_jid(&p)?),
        None => None,
    };
    session
        .client
        .chat_actions()
        .delete_message_for_me(&chat, participant_jid.as_ref(), &message_id, from_me, false, timestamp)
        .await
        .map_err(ApiError::library)?;
    Ok(())
}

/// Send read receipts for specific message ids (viewport-driven). For group
/// messages pass the message's sender; for DMs/status pass `None`.
#[tauri::command]
pub async fn mark_read_messages(
    jid: String,
    sender: Option<String>,
    message_ids: Vec<String>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    if message_ids.is_empty() {
        return Ok(());
    }
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let sender_jid = match sender {
        Some(s) => Some(parse_jid(&s)?),
        None => None,
    };
    let ids: Vec<&str> = message_ids.iter().map(String::as_str).collect();
    session
        .client
        .mark_as_read(&chat, sender_jid.as_ref(), &ids)
        .await
        .map_err(ApiError::library)
}

/// Mark a chat as read (clears the unread badge for the whole chat).
#[tauri::command]
pub async fn mark_read(
    jid: String,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    session
        .client
        .chat_actions()
        .mark_chat_as_read(&chat, true, None)
        .await
        .map_err(ApiError::library)
}
