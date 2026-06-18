//! Bridges the whatsapp-rust `Event` bus to Tauri events.
//!
//! Each library `Event` is re-emitted twice: once on a per-kind topic
//! (`wa://message`, `wa://auth/qr`, …) for handlers that only care about one
//! thing, and once on the catch-all `wa://event` so the dev UI can observe
//! everything. Every payload is wrapped with `sessionId` so a multi-session
//! frontend knows which account it came from.

use std::sync::Arc;

use async_channel::Receiver;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use whatsapp_rust::prelude::{Event, EventKind};

/// Catch-all topic every event is also emitted on.
const GLOBAL_TOPIC: &str = "wa://event";

pub async fn pump(app: AppHandle, session_id: String, rx: Receiver<Arc<Event>>) {
    log::info!("[{session_id}] event bridge started");
    while let Ok(event) = rx.recv().await {
        let kind = event.kind();

        let payload: Value = match event.as_ref() {
            Event::Message(msg, info) => json!({
                "info": info,
                "text": event.message_text(),
                "message": msg,
            }),
            // Flatten the pairing events to the documented {code, timeoutSecs}
            // shape. Without this they serialize externally-tagged
            // ({"PairingQrCode":{...}}), so the UI reads `payload.code` as
            // undefined and renders a QR of the literal text "undefined".
            Event::PairingQrCode { code, timeout } => json!({
                "code": code,
                "timeoutSecs": timeout.as_secs(),
            }),
            Event::PairingCode { code, timeout } => json!({
                "code": code,
                "timeoutSecs": timeout.as_secs(),
            }),
            // Not serializable / too noisy for the UI; skip entirely.
            Event::Notification(_) | Event::RawNode(_) => continue,
            // Everything else: externally-tagged JSON, e.g. {"Connected":null}
            // or {"Receipt":{...}}. The `kind` field below disambiguates.
            other => serde_json::to_value(other).unwrap_or(Value::Null),
        };

        let envelope = json!({
            "sessionId": session_id,
            "kind": format!("{kind:?}"),
            "payload": payload,
        });

        let topic = topic_for(kind);
        if topic != GLOBAL_TOPIC {
            let _ = app.emit(topic, envelope.clone());
        }
        let _ = app.emit(GLOBAL_TOPIC, envelope);
    }
    log::info!("[{session_id}] event bridge stopped");
}

/// Maps an event kind to its dedicated Tauri topic. Kinds without a dedicated
/// topic fall through to the catch-all only.
fn topic_for(kind: EventKind) -> &'static str {
    match kind {
        EventKind::PairingQrCode => "wa://auth/qr",
        EventKind::PairingCode => "wa://auth/pair-code",
        EventKind::PairSuccess => "wa://auth/paired",
        EventKind::PairError => "wa://auth/pair-error",
        EventKind::LoggedOut => "wa://auth/logged-out",
        EventKind::Connected | EventKind::Disconnected => "wa://conn/state",
        EventKind::Message => "wa://message",
        EventKind::Receipt => "wa://receipt",
        EventKind::Presence | EventKind::ChatPresence => "wa://presence",
        EventKind::GroupUpdate => "wa://group/update",
        EventKind::ContactUpdated | EventKind::PushNameUpdate => "wa://contact/update",
        EventKind::IncomingCall => "wa://call",
        EventKind::StreamError | EventKind::ConnectFailure | EventKind::TemporaryBan => {
            "wa://error/stream"
        }
        _ => GLOBAL_TOPIC,
    }
}
