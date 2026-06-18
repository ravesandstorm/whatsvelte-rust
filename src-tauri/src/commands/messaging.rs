//! Messaging commands.

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::Jid;

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
