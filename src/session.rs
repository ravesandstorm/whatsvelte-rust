//! Multi-session manager for WhatsApp Dioxus.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use whatsapp_rust::prelude::*;
use whatsapp_rust::wacore::types::events::ChannelEventHandler;

use crate::bridge;
use crate::error::{ApiError, ApiResult};

use crate::dto::UiEvent;

/// Session id used when the frontend doesn't specify one.
pub const DEFAULT_SESSION: &str = "default";

/// A single booted account: the background run-loop task plus its client.
pub struct Session {
    /// Aborted on drop so `reset` releases the SQLite handle before deleting it.
    pub run_task: JoinHandle<()>,
    pub client: Arc<Client>,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.run_task.abort();
    }
}

pub struct SessionManager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
    /// Directory where per-session SQLite files live.
    base_dir: PathBuf,
    event_tx: async_channel::Sender<(String, UiEvent)>,
    event_rx: async_channel::Receiver<(String, UiEvent)>,
}

impl SessionManager {
    pub fn new(base_dir: PathBuf) -> Self {
        let (event_tx, event_rx) = async_channel::unbounded();
        Self {
            sessions: Mutex::new(HashMap::new()),
            base_dir,
            event_tx,
            event_rx,
        }
    }

    pub fn event_receiver(&self) -> async_channel::Receiver<(String, UiEvent)> {
        self.event_rx.clone()
    }

    /// Normalize an optional id to a concrete session id.
    pub fn resolve_id(session_id: Option<String>) -> String {
        session_id
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SESSION.to_string())
    }

    /// Return the session, booting it if it isn't running yet.
    pub async fn get_or_create(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        let mut map = self.sessions.lock().await;
        if let Some(existing) = map.get(session_id) {
            if !existing.run_task.is_finished() {
                return Ok(existing.clone());
            }
            log::warn!("[{session_id}] session run task terminated; removing dead session");
            map.remove(session_id);
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

    /// Hard-reset a session.
    pub async fn reset(&self, session_id: &str) -> ApiResult<Arc<Session>> {
        {
            let mut map = self.sessions.lock().await;
            if let Some(old) = map.remove(session_id) {
                old.run_task.abort();
                old.client.disconnect().await;
                drop(old);
            }
        }
        // Give SQLite file descriptors and background thread pools time to flush & close
        tokio::time::sleep(Duration::from_millis(150)).await;

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
        if let Err(e) = std::fs::create_dir_all(&self.base_dir) {
            log::warn!("failed to create session directory {}: {e}", self.base_dir.display());
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

        let client = bot.client();
        let (handler, rx) = ChannelEventHandler::new();
        client.register_handler(handler);

        let dead_sid = session_id.to_string();
        let dead_client = client.clone();
        let event_tx_clone = self.event_tx.clone();
        let run_task = tokio::spawn(async move {
            bot.run().await;
            if dead_client.is_logged_in() {
                log::info!("[{dead_sid}] run loop ended (logged in)");
            } else {
                log::warn!("[{dead_sid}] run loop ended while unpaired; requesting session reset");
                let _ = event_tx_clone.send((dead_sid, UiEvent::LoggedOut)).await;
            }
        });

        let sid = session_id.to_string();
        tokio::spawn(bridge::pump(sid, rx, self.event_tx.clone()));

        Ok(Arc::new(Session { run_task, client }))
    }
}
