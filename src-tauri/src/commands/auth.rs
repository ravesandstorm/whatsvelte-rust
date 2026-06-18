//! Lifecycle & pairing commands.

use std::sync::Arc;

use tauri::State;
use whatsapp_rust::pair_code::PairCodeOptions;

use crate::dto::{PairCodeDto, StatusDto};
use crate::error::{ApiError, ApiResult};
use crate::session::SessionManager;

type Mgr<'a> = State<'a, Arc<SessionManager>>;

/// Boot the session if needed and report login/connection state. The QR code
/// (if not yet paired) arrives separately on the `wa://auth/qr` event.
#[tauri::command]
pub async fn auth_status(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<StatusDto> {
    let (id, session) = mgr.session(session_id).await?;
    Ok(StatusDto {
        session_id: id,
        logged_in: session.client.is_logged_in(),
        connected: session.client.is_connected(),
        jid: session.client.get_pn().map(|j| j.to_string()),
    })
}

/// Ensure the session is running so QR pairing starts. Idempotent. The code is
/// pushed via the `wa://auth/qr` event.
#[tauri::command]
pub async fn auth_start_qr(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<()> {
    mgr.session(session_id).await?;
    Ok(())
}

/// Request a phone-number pairing code (alternative to scanning the QR).
#[tauri::command]
pub async fn auth_start_pair_code(
    phone: String,
    custom_code: Option<String>,
    session_id: Option<String>,
    mgr: Mgr<'_>,
) -> ApiResult<PairCodeDto> {
    let (id, session) = mgr.session(session_id).await?;
    let code = session
        .client
        .pair_with_code(PairCodeOptions {
            phone_number: phone,
            custom_code,
            ..Default::default()
        })
        .await
        .map_err(ApiError::library)?;
    Ok(PairCodeDto {
        session_id: id,
        code,
    })
}

#[tauri::command]
pub async fn connect(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.connect().await.map_err(ApiError::library)
}

#[tauri::command]
pub async fn disconnect(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.disconnect().await;
    Ok(())
}

#[tauri::command]
pub async fn auth_logout(session_id: Option<String>, mgr: Mgr<'_>) -> ApiResult<()> {
    let (_, session) = mgr.session(session_id).await?;
    session.client.logout().await.map_err(ApiError::library)
}
