//! Lifecycle & pairing operations.

use whatsapp_rust::pair_code::PairCodeOptions;

use crate::dto::{PairCodeDto, StatusDto};
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

/// Boot the session if needed and report login/connection state.
pub async fn auth_status(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<StatusDto> {
    let (id, session) = mgr.session(session_id).await?;
    let device = session.client.persistence_manager().get_device_snapshot();
    let push_name = device.push_name.clone();
    Ok(StatusDto {
        session_id: id,
        logged_in: session.client.is_logged_in(),
        connected: session.client.is_connected(),
        jid: session.client.get_pn().map(|j| j.to_string()),
        registered: device.is_registered(),
        push_name: if push_name.is_empty() { None } else { Some(push_name) },
    })
}

/// Ensure the session is running so QR pairing starts.
pub async fn auth_start_qr(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<()> {
    mgr.session(session_id).await?;
    Ok(())
}

/// Request a phone-number pairing code.
pub async fn auth_start_pair_code(
    phone: String,
    custom_code: Option<String>,
    session_id: Option<String>,
    mgr: &SessionManager,
) -> ApiResult<PairCodeDto> {
    let (id, session) = mgr.session(session_id).await?;
    session
        .client
        .wait_for_socket(std::time::Duration::from_secs(30))
        .await
        .map_err(ApiError::library)?;
    let code = session
        .client
        .pair_with_code(PairCodeOptions {
            phone_number: phone,
            custom_code,
            ..Default::default()
        })
        .await
        .map_err(ApiError::source_chain)?;
    Ok(PairCodeDto {
        session_id: id,
        code,
    })
}

pub async fn connect(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.connect().await.map_err(ApiError::library)
}

pub async fn disconnect(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.disconnect().await;
    Ok(())
}

pub async fn auth_logout(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.logout().await.map_err(ApiError::library)
}

/// Hard-reset the on-disk session.
pub async fn reset_session(session_id: Option<String>, mgr: &SessionManager) -> ApiResult<()> {
    let id = SessionManager::resolve_id(session_id);
    mgr.reset(&id).await?;
    Ok(())
}
