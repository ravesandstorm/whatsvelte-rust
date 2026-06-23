//! Newsletter (channel) commands. The frontend has no other way to learn which
//! channels the account follows — they're not part of history sync and only
//! arrive live when a new post comes in. `list_newsletters` fetches the
//! subscribed set so the Channels section is populated on connect.

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::NewsletterMetadata;

use crate::dto::ChatDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

/// List the channels (newsletters) this account is subscribed to, as chat rows.
/// Live network call (MEX over the socket) — requires an active connection.
#[tauri::command]
pub async fn list_newsletters(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<Vec<ChatDto>> {
    let (_, session) = mgr.session(session_id).await?;
    let newsletters: Vec<NewsletterMetadata> = session
        .client
        .newsletter()
        .list_subscribed()
        .await
        .map_err(ApiError::library)?;

    Ok(newsletters
        .into_iter()
        .map(|meta| ChatDto {
            jid: meta.jid.to_string(),
            name: Some(meta.name),
            last_message: None,
            timestamp: meta.creation_time.map(|t| t as i64).unwrap_or(0),
            unread: 0,
        })
        .collect())
}
