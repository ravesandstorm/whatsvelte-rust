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

use tokio::sync::Mutex;
use whatsapp_rust::prelude::*;

use crate::core::bridge::{self, UiEvent};
use crate::core::error::{ApiError, ApiResult};

/// Session id used when the frontend doesn't specify one.
pub const DEFAULT_SESSION: &str = "default";

/// A single booted account: the background run-loop task plus its client.
pub struct Session {
    /// The run loop. `reset` takes it, aborts it and waits for it to finish, so
    /// the SQLite handle it owns is closed before the files are deleted.
    run_task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub client: Arc<Client>,
}

impl Session {
    fn take_run_task(&self) -> Option<tokio::task::JoinHandle<()>> {
        self.run_task
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(task) = self.take_run_task() {
            task.abort();
        }
    }
}

pub struct SessionManager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
    ui: async_channel::Sender<UiEvent>,
    /// Directory where per-session SQLite files live (dev: rust-backend/).
    base_dir: PathBuf,
}

impl SessionManager {
    pub fn new(ui: async_channel::Sender<UiEvent>, base_dir: PathBuf) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            ui,
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
        // Hold the map lock for the whole reset, so a command arriving meanwhile
        // can't boot a client on the half-deleted database.
        let mut map = self.sessions.lock().await;
        if let Some(old) = map.remove(session_id) {
            old.client.disconnect().await;
            // `abort` only requests cancellation; wait for the task to actually
            // end so it drops the bot (and with it the SQLite pool).
            if let Some(task) = old.take_run_task() {
                task.abort();
                let _ = task.await;
            }
            drop(old);
        }
        self.delete_db_files(session_id).await;
        let session = self.boot(session_id).await?;
        map.insert(session_id.to_string(), session.clone());
        Ok(session)
    }

    /// Delete the db plus its WAL/SHM sidecars; a missing file is not an error.
    /// Windows refuses to delete a file that still has an open handle (e.g. a
    /// command that was mid-query when the reset began), so retry briefly.
    async fn delete_db_files(&self, session_id: &str) {
        let paths: Vec<PathBuf> = ["", "-shm", "-wal"]
            .iter()
            .map(|suffix| {
                self.base_dir
                    .join(format!("whatsapp-{session_id}.db{suffix}"))
            })
            .collect();
        for attempt in 0..30 {
            let mut busy = false;
            for path in &paths {
                match std::fs::remove_file(path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        busy = true;
                        if attempt == 29 {
                            log::warn!("[{session_id}] failed to delete {}: {e}", path.display());
                        }
                    }
                }
            }
            if !busy {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn boot(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        // The debug demo (WA_DEMO) must never touch the network or a real
        // session database.
        if cfg!(debug_assertions) && std::env::var_os("WA_DEMO").is_some() {
            return Err(ApiError::NotConnected);
        }
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
        let (handler, rx) = bridge::BridgeHandler::new();
        client.register_handler(handler);

        // Drive the run loop ourselves (rather than `bot.spawn()`) so we can react
        // when it shuts down. Once all QR codes expire the library disconnects and
        // the loop exits for good — it won't restart itself, so if we're still
        // unpaired the on-screen QR is now dead and the app must be relaunched.
        // (An abort from `Session`'s drop cancels this task before the emit, so a
        // reset-triggered shutdown never reports as "dead".)
        let dead_ui = self.ui.clone();
        let dead_sid = session_id.to_string();
        let dead_client = client.clone();
        let run_task = tokio::spawn(async move {
            bot.run().await;
            if dead_client.is_logged_in() {
                log::info!("[{dead_sid}] run loop ended (logged in)");
            } else {
                log::warn!(
                    "[{dead_sid}] run loop ended while unpaired; QR is dead, restart required"
                );
                let _ = dead_ui
                    .send(UiEvent::ClientDead {
                        session_id: dead_sid,
                    })
                    .await;
            }
        });

        // Drain the library event stream into typed UI events for this session.
        let ui = self.ui.clone();
        let sid = session_id.to_string();
        tokio::spawn(bridge::pump(ui, sid, rx));

        Ok(Arc::new(Session {
            run_task: std::sync::Mutex::new(Some(run_task)),
            client,
        }))
    }
}
