//! Contact lookup operations.

use whatsapp_rust::Jid;

use crate::dto::{ContactDto, ResolveJidDto};
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

fn parse_jid(s: &str) -> ApiResult<Jid> {
    s.parse::<Jid>()
        .map_err(|e| ApiError::InvalidJid(format!("{s}: {e}")))
}

pub async fn get_contact(
    jid: String,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<ContactDto> {
    let (_, session) = mgr.session(session_id).await?;
    let parsed = parse_jid(&jid)?;

    let info = session
        .client
        .contacts()
        .get_user_info(std::slice::from_ref(&parsed))
        .await
        .ok()
        .and_then(|mut m| m.remove(&parsed));

    let verified_name = info
        .as_ref()
        .and_then(|u| u.verified_name.as_ref().and_then(|vn| vn.name.clone()));
    let lid = info.as_ref().and_then(|u| u.lid.as_ref().map(|l| l.to_string()));

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
        name: verified_name.clone(),
        verified_name,
        lid,
        picture_url,
    })
}

pub async fn resolve_jid(
    jid: String,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<ResolveJidDto> {
    let (_, session) = mgr.session(session_id).await?;
    let parsed = parse_jid(&jid)?;
    let entry = session
        .client
        .get_lid_pn_entry(&parsed)
        .await
        .map_err(ApiError::library)?;
    Ok(match entry {
        Some(e) => ResolveJidDto {
            pn: Some(format!("{}@s.whatsapp.net", e.phone_number)),
            lid: Some(format!("{}@lid", e.lid)),
        },
        None => ResolveJidDto { pn: None, lid: None },
    })
}

pub async fn get_profile_picture_url(
    jid: String,
    preview: Option<bool>,
    session_id: Option<String>,
    mgr: &SessionManager,
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
