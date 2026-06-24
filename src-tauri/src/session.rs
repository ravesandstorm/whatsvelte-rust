//! Multi-session manager.
//!
//! whatsapp-rust is single-account per `Client`/`SqliteStore`. To support
//! several linked accounts we keep one booted `Client` per `sessionId`, each
//! backed by its own SQLite file. For the current testing scope the frontend
//! can omit `sessionId` and everything falls back to [`DEFAULT_SESSION`] — but
//! the plumbing is real, so wiring more sessions later is a frontend-only
//! change.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::Emitter;
use tokio::sync::Mutex;
use whatsapp_rust::prelude::*;
use whatsapp_rust::wacore::types::events::ChannelEventHandler;

use crate::bridge;
use crate::error::{ApiError, ApiResult};

/// Session id used when the frontend doesn't specify one.
pub const DEFAULT_SESSION: &str = "default";

/// A single booted account: the background run-loop task plus its client.
pub struct Session {
    /// Aborted on drop so `reset` releases the SQLite handle before deleting it.
    run_task: tauri::async_runtime::JoinHandle<()>,
    pub client: Arc<Client>,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.run_task.abort();
    }
}

pub struct SessionManager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
    app: tauri::AppHandle,
    /// Directory where per-session SQLite files live (dev: rust-backend/).
    base_dir: PathBuf,
}

impl SessionManager {
    pub fn new(app: tauri::AppHandle, base_dir: PathBuf) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            app,
            base_dir,
        }
    }

    /// Normalize an optional id from the frontend to a concrete session id.
    pub fn resolve_id(session_id: Option<String>) -> String {
        session_id
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SESSION.to_string())
    }

    /// Return the session, booting it (which starts emitting QR/connection
    /// events) if it isn't running yet.
    pub async fn get_or_create(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        let mut map = self.sessions.lock().await;
        if let Some(existing) = map.get(session_id) {
            return Ok(existing.clone());
        }
        let session = self.boot(session_id).await?;
        map.insert(session_id.to_string(), session.clone());
        Ok(session)
    }

    /// Convenience: resolve an optional id and fetch/boot in one call.
    pub async fn session(&self, session_id: Option<String>) -> ApiResult<(String, Arc<Session>)> {
        let id = Self::resolve_id(session_id);
        let session = self.get_or_create(&id).await?;
        Ok((id, session))
    }

    /// Hard-reset a session: drop the live client, delete its SQLite files so
    /// the device key is regenerated, then boot a fresh (unregistered) session
    /// that will emit a new QR. Used by logout and by stale-data recovery — if
    /// the on-disk key is stale the QR never generates, so we wipe and recreate.
    pub async fn reset(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        {
            let mut map = self.sessions.lock().await;
            if let Some(old) = map.remove(session_id) {
                old.client.disconnect().await;
                // Dropping the map's strong ref aborts the run loop (BotHandle in
                // Session), releasing the SQLite handle before we delete the file.
                drop(old);
            }
        }
        // Delete the db plus its WAL/SHM sidecars; a missing file is not an error.
        for suffix in ["", "-shm", "-wal"] {
            let path = self.base_dir.join(format!("whatsapp-{session_id}.db{suffix}"));
            if let Err(e) = std::fs::remove_file(&path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::warn!("[{session_id}] failed to delete {}: {e}", path.display());
                }
            }
        }
        let session = self.boot(session_id).await?;
        self.sessions
            .lock()
            .await
            .insert(session_id.to_string(), session.clone());
        Ok(session)
    }

    async fn boot(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        let db_path = self.base_dir.join(format!("whatsapp-{session_id}.db"));
        let db_str = db_path.to_string_lossy().to_string();
        log::info!("[{session_id}] booting session, db = {db_str}");

        let store = SqliteStore::new(&db_str).await.map_err(ApiError::library)?;
        let bot = Bot::builder()
            .with_backend(store)
            .build()
            .await
            .map_err(ApiError::library)?;

        // Register the event bridge BEFORE spawning the run loop, so the QR /
        // pairing events emitted right after connect are never missed.
        let client = bot.client();
        let (handler, rx) = ChannelEventHandler::new();
        client.register_handler(handler);

        // Drive the run loop ourselves (rather than `bot.spawn()`) so we can react
        // when it shuts down. Once all QR codes expire the library disconnects and
        // the loop exits for good — it won't restart itself, so if we're still
        // unpaired the on-screen QR is now dead and the app must be relaunched.
        // (An abort from `Session`'s drop cancels this task before the emit, so a
        // reset-triggered shutdown never reports as "dead".)
        let dead_app = self.app.clone();
        let dead_sid = session_id.to_string();
        let dead_client = client.clone();
        let run_task = tauri::async_runtime::spawn(async move {
            bot.run().await;
            if dead_client.is_logged_in() {
                log::info!("[{dead_sid}] run loop ended (logged in)");
            } else {
                log::warn!("[{dead_sid}] run loop ended while unpaired; QR is dead, restart required");
                let envelope = json!({ "sessionId": dead_sid, "kind": "ClientDead", "payload": {} });
                let _ = dead_app.emit("wa://auth/dead", envelope.clone());
                let _ = dead_app.emit("wa://event", envelope);
            }
        });

        // Drain the library event stream into Tauri events for this session.
        let app = self.app.clone();
        let sid = session_id.to_string();
        tauri::async_runtime::spawn(bridge::pump(app, sid, rx));

        Ok(Arc::new(Session { run_task, client }))
    }
}
