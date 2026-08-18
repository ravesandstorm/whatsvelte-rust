//! Server-synced chat settings (mute / pin / archive).

use whatsapp_rust::Jid;

use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

pub async fn set_chat_muted(
    jid: String,
    muted: bool,
    session_id: Option<String>,
    mgr: &SessionManager,
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

pub async fn set_chat_pinned(
    jid: String,
    pinned: bool,
    session_id: Option<String>,
    mgr: &SessionManager,
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

pub async fn set_chat_archived(
    jid: String,
    archived: bool,
    session_id: Option<String>,
    mgr: &SessionManager,
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
