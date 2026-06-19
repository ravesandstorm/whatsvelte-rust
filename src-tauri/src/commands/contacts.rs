//! Contact lookup commands (lazy, pull-on-demand).
//!
//! Display names for regular users are not server-queryable — they come from
//! history-sync conversation names and message pushNames. These commands mainly
//! provide profile-picture URLs and verified business names to enrich the UI.

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::Jid;

use crate::dto::ContactDto;
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

#[tauri::command]
pub async fn get_contact(
    jid: String,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<ContactDto> {
    let (_, session) = mgr.session(session_id).await?;
    let parsed = parse_jid(&jid)?;

    // Only verified businesses expose a name here; regular names come from the
    // event/history stream. Best-effort: don't fail the whole call on lookup error.
    let name = session
        .client
        .contacts()
        .get_user_info(std::slice::from_ref(&parsed))
        .await
        .ok()
        .and_then(|m| {
            m.get(&parsed)
                .and_then(|u| u.verified_name.as_ref().and_then(|vn| vn.name.clone()))
        });

    let picture_url = session
        .client
        .contacts()
        .get_profile_picture(&parsed, true)
        .await
        .ok()
        .flatten()
        .map(|p| p.url);

    Ok(ContactDto {
        jid,
        name,
        picture_url,
    })
}

#[tauri::command]
pub async fn get_profile_picture_url(
    jid: String,
    preview: Option<bool>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<Option<String>> {
    let (_, session) = mgr.session(session_id).await?;
    let parsed = parse_jid(&jid)?;
    let url = session
        .client
        .contacts()
        .get_profile_picture(&parsed, preview.unwrap_or(true))
        .await
        .map_err(ApiError::library)?
        .map(|p| p.url);
    Ok(url)
}
