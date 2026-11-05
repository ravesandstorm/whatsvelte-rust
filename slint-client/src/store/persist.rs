//! On-disk UI cache — replaces the WebView's IndexedDB + localStorage.
//!
//! Layout under `<app-data>/ui-cache/`:
//!
//! ```text
//! account.json                  { "jid": "<self pn jid>" }
//! chats.json                    chat rows (flags, drafts, unread)
//! names.json                    learned push names, jid -> name
//! lid.json                      learned LID -> PN map
//! msgs/<jid>.json               full message history of one chat
//! ```
//!
//! Every write goes to a `.tmp` sibling first and is renamed into place, so a
//! crash mid-write never leaves a truncated file behind.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::{Chat, UiMessage};

pub struct Persist {
    root: PathBuf,
}

impl Persist {
    pub fn new(root: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(root.join("msgs"));
        Self { root }
    }

    fn read<T: DeserializeOwned>(path: &Path) -> Option<T> {
        let bytes = std::fs::read(path).ok()?;
        match serde_json::from_slice(&bytes) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("ignoring corrupt cache file {}: {e}", path.display());
                None
            }
        }
    }

    fn write<T: Serialize + ?Sized>(path: &Path, value: &T) {
        let tmp = path.with_extension("json.tmp");
        let result = serde_json::to_vec(value)
            .map_err(std::io::Error::other)
            .and_then(|bytes| std::fs::write(&tmp, bytes))
            .and_then(|()| std::fs::rename(&tmp, path));
        if let Err(e) = result {
            let _ = std::fs::remove_file(&tmp);
            log::error!("failed to write {}: {e}", path.display());
        }
    }

    /// Filesystem-safe file name for a chat jid (`@`, `:` and `.` survive on
    /// every OS we target except `:` on Windows, so map it).
    fn msgs_path(&self, jid: &str) -> PathBuf {
        let safe: String = jid
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '-' | '_') { c } else { '_' })
            .collect();
        self.root.join("msgs").join(format!("{safe}.json"))
    }

    pub fn load_account(&self) -> Option<String> {
        #[derive(serde::Deserialize)]
        struct A {
            jid: String,
        }
        Self::read::<A>(&self.root.join("account.json")).map(|a| a.jid)
    }

    pub fn save_account(&self, jid: &str) {
        Self::write(&self.root.join("account.json"), &serde_json::json!({ "jid": jid }));
    }

    pub fn load_chats(&self) -> Vec<Chat> {
        Self::read(&self.root.join("chats.json")).unwrap_or_default()
    }

    pub fn save_chats(&self, chats: &[&Chat]) {
        Self::write(&self.root.join("chats.json"), chats);
    }

    pub fn load_map(&self, name: &str) -> HashMap<String, String> {
        Self::read(&self.root.join(format!("{name}.json"))).unwrap_or_default()
    }

    pub fn save_map(&self, name: &str, map: &HashMap<String, String>) {
        Self::write(&self.root.join(format!("{name}.json")), map);
    }

    pub fn load_messages(&self, jid: &str) -> Vec<UiMessage> {
        Self::read(&self.msgs_path(jid)).unwrap_or_default()
    }

    pub fn save_messages(&self, jid: &str, msgs: &[UiMessage]) {
        Self::write(&self.msgs_path(jid), msgs);
    }

    pub fn delete_messages(&self, jid: &str) {
        let _ = std::fs::remove_file(self.msgs_path(jid));
    }

    /// Wipe everything (logout / different account linked).
    pub fn clear_all(&self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::create_dir_all(self.root.join("msgs"));
    }
}
