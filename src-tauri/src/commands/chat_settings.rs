//! Server-synced chat settings (mute / pin / archive). Thin wrappers over
//! `client.chat_actions()`; the resulting app-state change is echoed back as a
//! `MuteUpdate`/`PinUpdate`/`ArchiveUpdate` event (see bridge `wa://chat/flags`).

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::Jid;

use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

#[tauri::command]
pub async fn set_chat_muted(
    jid: String,
    muted: bool,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let actions = session.client.chat_actions();
    if muted {
        actions.mute_chat(&chat).await
    } else {
        actions.unmute_chat(&chat).await
    }
    .map_err(ApiError::library)
}

#[tauri::command]
pub async fn set_chat_pinned(
    jid: String,
    pinned: bool,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let actions = session.client.chat_actions();
    if pinned {
        actions.pin_chat(&chat).await
    } else {
        actions.unpin_chat(&chat).await
    }
    .map_err(ApiError::library)
}

#[tauri::command]
pub async fn set_chat_archived(
    jid: String,
    archived: bool,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    let chat = parse_jid(&jid)?;
    let actions = session.client.chat_actions();
    if archived {
        actions.archive_chat(&chat, None).await
    } else {
        actions.unarchive_chat(&chat, None).await
    }
    .map_err(ApiError::library)
}
