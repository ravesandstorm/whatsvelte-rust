//! Newsletter (channel) operations.

use whatsapp_rust::NewsletterMetadata;

use crate::dto::ChatDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

pub async fn list_newsletters(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<Vec<ChatDto>> {
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
