//! UI state: chat rows, per-chat message windows, learned names and the
//! LID ↔ PN map. Port of `svelte-frontend/src/lib/stores/*` + `persist.ts`.
//!
//! Memory model (same as the Svelte build): the full history lives on disk.
//! In memory each chat holds either its whole history (`full`, the open chat or
//! one being mutated) or only its newest [`RESIDENT_TAIL`] messages. Anything
//! that needs an older message calls [`Store::ensure_full`], which merges the
//! on-disk copy in. Dirty chats are written back by [`Store::flush`], after
//! which non-open chats are trimmed back to their tail.

pub mod persist;
pub mod settings;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::core::dto::{ChatDto, ChatFlagsDto, MessageDto, QuotedDto};
use crate::util::jid::{is_lid, is_newsletter, is_regular_chat, is_status, normalize_jid};
use crate::util::time::now_secs;
use persist::Persist;

/// Messages kept in memory for chats that aren't open.
pub const RESIDENT_TAIL: usize = 20;
/// The open chat's history is rewritten at most this often: reading a long chat
/// dirties it constantly, and saving holds the store lock.
const ACTIVE_SAVE_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    #[default]
    None,
    Failed,
    Sending,
    Sent,
    Delivered,
    Read,
    Played,
}

impl MessageStatus {
    pub fn parse(s: &str) -> Self {
        match s {
            "sent" => Self::Sent,
            "delivered" => Self::Delivered,
            "read" => Self::Read,
            "played" => Self::Played,
            _ => Self::None,
        }
    }

    /// Monotonic ordering so a late receipt can't downgrade status.
    fn rank(self) -> u8 {
        match self {
            Self::None | Self::Failed | Self::Sending => 0,
            Self::Sent => 1,
            Self::Delivered => 2,
            Self::Read => 3,
            Self::Played => 4,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiMessage {
    #[serde(flatten)]
    pub dto: MessageDto,
    #[serde(default)]
    pub status: MessageStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<i64>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub deleted: bool,
    /// Reactor jid -> emoji.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reactions: BTreeMap<String, String>,
    /// Addressing as the server used it, when it differs from the canonical
    /// (LID → PN) keys. Protocol actions on the message must use these.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wire_chat_jid: Option<String>,
    /// Incoming and not yet seen (drives the unread divider + read receipts).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unread: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wire_sender_jid: Option<String>,
}

impl UiMessage {
    pub fn is_pending(&self) -> bool {
        self.dto.id.starts_with("tmp-")
            || matches!(self.status, MessageStatus::Sending | MessageStatus::Failed)
    }
    pub fn proto_chat_jid(&self) -> &str {
        self.wire_chat_jid.as_deref().unwrap_or(&self.dto.chat_jid)
    }
    pub fn proto_sender_jid(&self) -> &str {
        self.wire_sender_jid
            .as_deref()
            .unwrap_or(&self.dto.sender_jid)
    }
}

/// Chat-list / quote label for a message kind (port of preview.ts mediaLabel).
pub fn media_label(kind: &str) -> &'static str {
    match kind {
        "image" => "📷 Photo",
        "video" => "🎥 Video",
        "audio" => "🎙 Audio",
        "document" => "📄 Document",
        "sticker" => "🃏 Sticker",
        "buttons" | "list" | "interactive" | "template" => "📋 Interactive",
        "poll" => "📊 Poll",
        "order" => "🛒 Order",
        "product" => "🛍 Product",
        "contact" => "👤 Contact",
        "location" => "📍 Location",
        _ => "Message",
    }
}

/// Quoted-preview text (port of preview.ts formatQuotedText).
pub fn quoted_text(kind: &str, text: Option<&str>, file_name: Option<&str>) -> String {
    match text.map(str::trim).filter(|t| !t.is_empty()) {
        Some(t) if kind == "text" => t.to_string(),
        Some(t) => format!("{} {t}", media_label(kind)),
        None => match file_name {
            Some(n) => format!("📄 {n}"),
            None => media_label(kind).to_string(),
        },
    }
}

/// A message with nothing to draw (protocol traffic).
pub fn is_blank(m: &MessageDto) -> bool {
    m.kind == "other" && m.text.is_none() && m.media.is_none() && m.thumbnail.is_none()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub jid: String,
    pub name: Option<String>,
    pub last_message: Option<String>,
    pub timestamp: i64,
    pub unread: u32,
    #[serde(default)]
    pub muted: Option<bool>,
    #[serde(default)]
    pub pinned: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<String>,
}

impl Chat {
    pub fn is_muted(&self) -> bool {
        self.muted.unwrap_or(false)
    }
    pub fn is_pinned(&self) -> bool {
        self.pinned.unwrap_or(false)
    }
    pub fn is_archived(&self) -> bool {
        self.archived.unwrap_or(false)
    }
}

/// Which list the sidebar shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Chats,
    Archived,
    Channels,
    Status,
}

#[derive(Default)]
struct ChatMsgs {
    msgs: Vec<UiMessage>,
    /// True when `msgs` holds the chat's whole persisted history.
    full: bool,
}

pub struct Store {
    persist: Option<Persist>,
    pub chats: HashMap<String, Chat>,
    msgs: HashMap<String, ChatMsgs>,
    pub names: HashMap<String, String>,
    lid_to_pn: HashMap<String, String>,
    pub active: Option<String>,
    /// Receipts for ids we don't know yet (our own send, acked before the send
    /// call returned). Applied when the optimistic bubble is confirmed.
    early_receipts: HashMap<String, MessageStatus>,
    dirty_msgs: HashSet<String>,
    dirty_chats: bool,
    dirty_names: bool,
    dirty_lid: bool,
    tmp_seq: u64,
    active_saved: Option<Instant>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            persist: None,
            chats: HashMap::new(),
            msgs: HashMap::new(),
            names: HashMap::new(),
            lid_to_pn: HashMap::new(),
            active: None,
            early_receipts: HashMap::new(),
            dirty_msgs: HashSet::new(),
            dirty_chats: false,
            dirty_names: false,
            dirty_lid: false,
            tmp_seq: 0,
            active_saved: None,
        }
    }

    // ---------------------------------------------------------------- lifecycle

    /// Open the on-disk cache. Returns the account jid the cache belongs to.
    pub fn open_cache(&mut self, persist: Persist) -> Option<String> {
        let account = persist.load_account();
        self.persist = Some(persist);
        account
    }

    /// Rehydrate chat rows, names and the LID map. Returns the chat jids whose
    /// newest messages should be loaded next ([`Store::hydrate_tails`]).
    pub fn hydrate_rows(&mut self) -> Vec<String> {
        let Some(p) = self.persist.as_ref() else {
            return Vec::new();
        };
        self.names = p.load_map("names");
        self.lid_to_pn = p.load_map("lid");
        let rows = p.load_chats();
        for c in rows {
            let jid = self.canonical(&c.jid);
            if jid != c.jid {
                // A persisted LID chat whose mapping is known: re-key its history.
                let msgs = p.load_messages(&c.jid);
                if !msgs.is_empty() {
                    let mut merged = p.load_messages(&jid);
                    let ids: HashSet<String> = merged.iter().map(|m| m.dto.id.clone()).collect();
                    for mut m in msgs.into_iter().filter(|m| !ids.contains(&m.dto.id)) {
                        m.wire_chat_jid
                            .get_or_insert_with(|| m.dto.chat_jid.clone());
                        m.dto.chat_jid = jid.clone();
                        merged.push(m);
                    }
                    merged.sort_by_key(|m| m.dto.timestamp);
                    p.save_messages(&jid, &merged);
                    p.delete_messages(&c.jid);
                }
                self.dirty_chats = true;
            }
            let row = Chat {
                jid: jid.clone(),
                ..c
            };
            match self.chats.get(&jid) {
                Some(prev) if prev.timestamp > row.timestamp => {}
                _ => {
                    self.chats.insert(jid, row);
                }
            }
        }
        self.chats.keys().cloned().collect()
    }

    /// Load the newest message of each chat (previews + ticks); the rest of the
    /// history pages in when a chat is opened.
    pub fn hydrate_tails(&mut self, jids: &[String]) {
        let Some(p) = self.persist.as_ref() else {
            return;
        };
        for jid in jids {
            let all = p.load_messages(jid);
            let start = all.len().saturating_sub(1);
            self.msgs.insert(
                jid.clone(),
                ChatMsgs {
                    msgs: all[start..].to_vec(),
                    full: false,
                },
            );
        }
    }

    pub fn set_account(&mut self, jid: &str) {
        if let Some(p) = &self.persist {
            p.save_account(jid);
        }
    }

    /// Forget everything, in memory and on disk.
    pub fn clear_all(&mut self) {
        self.chats.clear();
        self.msgs.clear();
        self.names.clear();
        self.lid_to_pn.clear();
        self.active = None;
        self.early_receipts.clear();
        self.dirty_msgs.clear();
        self.dirty_chats = false;
        self.dirty_names = false;
        self.dirty_lid = false;
        if let Some(p) = &self.persist {
            p.clear_all();
        }
    }

    /// Periodic flush: like [`Store::flush`], but the open chat's history is
    /// left dirty until [`ACTIVE_SAVE_INTERVAL`] has passed.
    pub fn flush_periodic(&mut self) {
        let due = self
            .active_saved
            .is_none_or(|t| t.elapsed() >= ACTIVE_SAVE_INTERVAL);
        let held = self
            .active
            .clone()
            .filter(|jid| !due && self.dirty_msgs.remove(jid));
        self.flush();
        if let Some(jid) = held {
            self.dirty_msgs.insert(jid);
        }
    }

    /// Write every dirty piece back to disk, then trim non-open chats.
    pub fn flush(&mut self) {
        let Some(p) = self.persist.as_ref() else {
            return;
        };
        if self.dirty_chats {
            let rows: Vec<&Chat> = self.chats.values().collect();
            p.save_chats(&rows);
            self.dirty_chats = false;
        }
        if self.dirty_names {
            p.save_map("names", &self.names);
            self.dirty_names = false;
        }
        if self.dirty_lid {
            p.save_map("lid", &self.lid_to_pn);
            self.dirty_lid = false;
        }
        let dirty: Vec<String> = self.dirty_msgs.drain().collect();
        for jid in dirty {
            self.ensure_full(&jid);
            if let Some(cm) = self.msgs.get(&jid) {
                self.persist.as_ref().unwrap().save_messages(&jid, &cm.msgs);
            }
            if self.active.as_deref() == Some(&jid) {
                self.active_saved = Some(Instant::now());
            }
        }
        self.trim_inactive();
    }

    fn trim_inactive(&mut self) {
        let active = self.active.clone();
        for (jid, cm) in self.msgs.iter_mut() {
            if Some(jid) != active.as_ref()
                && cm.msgs.len() > RESIDENT_TAIL
                && !self.dirty_msgs.contains(jid)
            {
                cm.msgs.drain(..cm.msgs.len() - RESIDENT_TAIL);
                cm.full = false;
            }
        }
    }

    /// Merge the on-disk history of `jid` into memory (memory copies win).
    pub fn ensure_full(&mut self, jid: &str) {
        let full = self.msgs.get(jid).is_some_and(|c| c.full);
        if full {
            return;
        }
        let disk = self
            .persist
            .as_ref()
            .map(|p| p.load_messages(jid))
            .unwrap_or_default();
        let cm = self.msgs.entry(jid.to_string()).or_default();
        let mem_ids: HashSet<String> = cm.msgs.iter().map(|m| m.dto.id.clone()).collect();
        let mut merged: Vec<UiMessage> = disk
            .into_iter()
            .filter(|m| !mem_ids.contains(&m.dto.id))
            .collect();
        merged.append(&mut cm.msgs);
        merged.sort_by_key(|m| m.dto.timestamp);
        cm.msgs = merged;
        cm.full = true;
    }

    // ---------------------------------------------------------------- queries

    pub fn messages(&self, jid: &str) -> &[UiMessage] {
        self.msgs.get(jid).map(|c| c.msgs.as_slice()).unwrap_or(&[])
    }

    pub fn find_message(&self, jid: &str, id: &str) -> Option<&UiMessage> {
        self.messages(jid).iter().find(|m| m.dto.id == id)
    }

    pub fn chat_list(&self, section: Section, query: &str) -> Vec<&Chat> {
        let q = query.trim().to_lowercase();
        let mut out: Vec<&Chat> = self
            .chats
            .values()
            .filter(|c| match section {
                Section::Chats => !c.is_archived() && is_regular_chat(&c.jid),
                Section::Archived => c.is_archived(),
                Section::Channels => !c.is_archived() && is_newsletter(&c.jid),
                Section::Status => !c.is_archived() && is_status(&c.jid),
            })
            .filter(|c| {
                q.is_empty()
                    || self.title_for(&c.jid).to_lowercase().contains(&q)
                    || c.jid.contains(&q)
                    || c.last_message
                        .as_deref()
                        .is_some_and(|m| m.to_lowercase().contains(&q))
            })
            .collect();
        out.sort_by(|a, b| {
            let pin = if section == Section::Chats {
                b.is_pinned().cmp(&a.is_pinned())
            } else {
                std::cmp::Ordering::Equal
            };
            pin.then(b.timestamp.cmp(&a.timestamp))
        });
        out
    }

    pub fn section_counts(&self) -> (usize, usize, usize) {
        let mut archived = 0;
        let mut channels = 0;
        let mut status = 0;
        for c in self.chats.values() {
            if c.is_archived() {
                archived += 1;
            } else if is_newsletter(&c.jid) {
                channels += 1;
            } else if is_status(&c.jid) {
                status += 1;
            }
        }
        (archived, channels, status)
    }

    /// Best human label for a jid: saved/history name, learned push name, phone.
    pub fn title_for(&self, jid: &str) -> String {
        let name = self.chats.get(jid).and_then(|c| c.name.as_deref());
        let push = self.names.get(jid).map(String::as_str);
        let shown = if is_lid(jid) {
            self.lid_to_pn.get(jid).map(String::as_str).unwrap_or(jid)
        } else {
            jid
        };
        crate::util::jid::display_name(shown, name, push)
    }

    // ---------------------------------------------------------------- LID ↔ PN

    /// Map a jid to its canonical (PN) form if a mapping is known.
    pub fn canonical(&self, jid: &str) -> String {
        if !is_lid(jid) {
            return jid.to_string();
        }
        self.lid_to_pn
            .get(jid)
            .cloned()
            .unwrap_or_else(|| jid.to_string())
    }

    pub fn knows_lid(&self, lid: &str) -> bool {
        self.lid_to_pn.contains_key(lid)
    }

    /// Record a LID → PN pair and merge any conversation split under the LID.
    pub fn learn_lid(&mut self, lid: &str, pn: &str) {
        let l = normalize_jid(lid);
        let p = normalize_jid(pn);
        if !is_lid(&l) || l == p {
            return;
        }
        if self.lid_to_pn.get(&l) != Some(&p) {
            self.lid_to_pn.insert(l.clone(), p.clone());
            self.dirty_lid = true;
        }
        self.merge_chat(&l, &p);
    }

    /// Merge chat `from` (row + history) into `to`.
    fn merge_chat(&mut self, from: &str, to: &str) {
        let has_row = self.chats.contains_key(from);
        let has_msgs = self.msgs.contains_key(from);
        if !has_row && !has_msgs {
            return;
        }
        if let Some(src) = self.chats.remove(from) {
            let merged = match self.chats.remove(to) {
                Some(dst) => {
                    let newer = if src.timestamp >= dst.timestamp {
                        &src
                    } else {
                        &dst
                    };
                    Chat {
                        jid: to.to_string(),
                        name: dst.name.clone().or(src.name.clone()),
                        last_message: newer.last_message.clone(),
                        timestamp: src.timestamp.max(dst.timestamp),
                        unread: src.unread + dst.unread,
                        muted: dst.muted.or(src.muted),
                        pinned: dst.pinned.or(src.pinned),
                        archived: dst.archived.or(src.archived),
                        draft: dst.draft.clone().or(src.draft.clone()),
                    }
                }
                None => Chat {
                    jid: to.to_string(),
                    ..src
                },
            };
            self.chats.insert(to.to_string(), merged);
            self.dirty_chats = true;
        }
        if has_msgs || self.persist.is_some() {
            self.ensure_full(from);
            self.ensure_full(to);
            let src = self.msgs.remove(from).unwrap_or_default();
            let dst = self.msgs.entry(to.to_string()).or_default();
            let ids: HashSet<String> = dst.msgs.iter().map(|m| m.dto.id.clone()).collect();
            for mut m in src.msgs {
                if ids.contains(&m.dto.id) {
                    continue;
                }
                if m.wire_chat_jid.is_none() {
                    m.wire_chat_jid = Some(m.dto.chat_jid.clone());
                }
                m.dto.chat_jid = to.to_string();
                dst.msgs.push(m);
            }
            dst.msgs.sort_by_key(|m| m.dto.timestamp);
            if let Some(p) = &self.persist {
                p.delete_messages(from);
            }
            self.dirty_msgs.remove(from);
            self.dirty_msgs.insert(to.to_string());
        }
        if self.active.as_deref() == Some(from) {
            self.active = Some(to.to_string());
        }
    }

    /// Re-key a message to its canonical chat/sender, remembering the wire form.
    pub fn canonical_message(&self, m: MessageDto) -> UiMessage {
        let chat = self.canonical(&m.chat_jid);
        let sender = self.canonical(&m.sender_jid);
        let wire_chat_jid = (chat != m.chat_jid).then(|| m.chat_jid.clone());
        let wire_sender_jid = (sender != m.sender_jid).then(|| m.sender_jid.clone());
        UiMessage {
            dto: MessageDto {
                chat_jid: chat,
                sender_jid: sender,
                ..m
            },
            wire_chat_jid,
            wire_sender_jid,
            ..Default::default()
        }
    }

    // ---------------------------------------------------------------- names

    pub fn learn_name(&mut self, jid: &str, name: &str) {
        if name.is_empty() || jid.is_empty() {
            return;
        }
        let jid = normalize_jid(jid);
        if self.names.get(&jid).map(String::as_str) != Some(name) {
            self.names.insert(jid, name.to_string());
            self.dirty_names = true;
        }
    }

    // ---------------------------------------------------------------- chats

    pub fn upsert_chat_dto(&mut self, d: &ChatDto) {
        let jid = self.canonical(&d.jid);
        let prev = self.chats.get(&jid).cloned();
        let newer = prev.as_ref().is_none_or(|p| d.timestamp >= p.timestamp);
        let row = Chat {
            jid: jid.clone(),
            name: d
                .name
                .clone()
                .or(prev.as_ref().and_then(|p| p.name.clone())),
            last_message: if newer {
                d.last_message
                    .clone()
                    .or(prev.as_ref().and_then(|p| p.last_message.clone()))
            } else {
                prev.as_ref().and_then(|p| p.last_message.clone())
            },
            timestamp: d.timestamp.max(prev.as_ref().map_or(0, |p| p.timestamp)),
            unread: if d.unread > 0 {
                d.unread
            } else {
                prev.as_ref().map_or(0, |p| p.unread)
            },
            muted: prev.as_ref().and_then(|p| p.muted),
            pinned: prev.as_ref().and_then(|p| p.pinned),
            archived: prev.as_ref().and_then(|p| p.archived),
            draft: prev.as_ref().and_then(|p| p.draft.clone()),
        };
        self.chats.insert(jid, row);
        self.dirty_chats = true;
    }

    fn touch_chat(&mut self, jid: &str, preview: Option<String>, ts: i64, incoming: bool) {
        let active = self.active.as_deref() == Some(jid);
        let c = self.chats.entry(jid.to_string()).or_insert_with(|| Chat {
            jid: jid.to_string(),
            ..Default::default()
        });
        if ts >= c.timestamp {
            if preview.is_some() {
                c.last_message = preview;
            }
            c.timestamp = ts;
        }
        let _ = active;
        if incoming {
            c.unread += 1;
        }
        self.dirty_chats = true;
    }

    pub fn apply_chat_flags(&mut self, f: &ChatFlagsDto) {
        let jid = self.canonical(&f.jid);
        if let Some(c) = self.chats.get_mut(&jid) {
            c.muted = f.muted.or(c.muted);
            c.pinned = f.pinned.or(c.pinned);
            c.archived = f.archived.or(c.archived);
            self.dirty_chats = true;
        }
    }

    pub fn set_flag(&mut self, jid: &str, flag: &str, value: bool) {
        if let Some(c) = self.chats.get_mut(jid) {
            match flag {
                "muted" => c.muted = Some(value),
                "pinned" => c.pinned = Some(value),
                "archived" => c.archived = Some(value),
                _ => return,
            }
            self.dirty_chats = true;
        }
    }

    pub fn set_draft(&mut self, jid: &str, text: &str) {
        let draft = (!text.trim().is_empty()).then(|| text.to_string());
        match self.chats.get_mut(jid) {
            Some(c) if c.draft != draft => c.draft = draft,
            Some(_) => return,
            None if draft.is_some() => {
                self.chats.insert(
                    jid.to_string(),
                    Chat {
                        jid: jid.to_string(),
                        draft,
                        ..Default::default()
                    },
                );
            }
            None => return,
        }
        self.dirty_chats = true;
    }

    pub fn open_chat(&mut self, jid: &str) {
        self.active = Some(jid.to_string());
        self.ensure_full(jid);
        if !self.chats.contains_key(jid) {
            self.chats.insert(
                jid.to_string(),
                Chat {
                    jid: jid.to_string(),
                    timestamp: now_secs(),
                    ..Default::default()
                },
            );
            self.dirty_chats = true;
        }
        self.apply_unread_flags(jid);
        self.trim_inactive();
    }

    /// Flag the last `chat.unread` incoming messages unread and the rest read.
    fn apply_unread_flags(&mut self, jid: &str) {
        let count = self.chats.get(jid).map_or(0, |c| c.unread as usize);
        let Some(cm) = self.msgs.get_mut(jid) else {
            return;
        };
        let mut left = count;
        let mut changed = false;
        for m in cm.msgs.iter_mut().rev() {
            let unread = !m.dto.from_me && !m.deleted && left > 0;
            if !m.dto.from_me && !m.deleted && left > 0 {
                left -= 1;
            }
            if m.unread != unread {
                m.unread = unread;
                changed = true;
            }
        }
        if changed {
            self.dirty_msgs.insert(jid.to_string());
        }
    }

    /// A message scrolled into view. Returns `Some(chat_now_read)` when it was
    /// unread (the caller then queues a read receipt, and marks the whole chat
    /// read once the counter reaches zero).
    pub fn mark_message_read(&mut self, jid: &str, id: &str) -> Option<bool> {
        let m = self
            .msgs
            .get_mut(jid)?
            .msgs
            .iter_mut()
            .find(|m| m.dto.id == id && m.unread)?;
        m.unread = false;
        self.dirty_msgs.insert(jid.to_string());
        let c = self.chats.get_mut(jid)?;
        if c.unread > 0 {
            c.unread -= 1;
            self.dirty_chats = true;
            return Some(c.unread == 0);
        }
        Some(false)
    }

    /// Put back a message snapshot (rollback of a failed optimistic action).
    pub fn restore_message(&mut self, m: UiMessage) {
        let jid = m.dto.chat_jid.clone();
        self.ensure_full(&jid);
        let cm = self.msgs.entry(jid.clone()).or_default();
        match cm.msgs.iter_mut().find(|x| x.dto.id == m.dto.id) {
            Some(x) => *x = m,
            None => {
                let pos = cm
                    .msgs
                    .partition_point(|x| x.dto.timestamp <= m.dto.timestamp);
                cm.msgs.insert(pos, m);
            }
        }
        self.dirty_msgs.insert(jid.clone());
        self.refresh_preview(&jid);
    }

    // ---------------------------------------------------------------- messages

    fn preview_of(m: &MessageDto) -> Option<String> {
        if let Some(t) = m.text.as_ref().filter(|t| !t.is_empty()) {
            return Some(t.clone());
        }
        Some(media_label(&m.kind).to_string())
    }

    /// Ingest a live message. Returns false when it was a duplicate.
    pub fn add_message(&mut self, mut m: UiMessage) -> bool {
        if is_blank(&m.dto) {
            return false;
        }
        let jid = m.dto.chat_jid.clone();
        let cm = self.msgs.entry(jid.clone()).or_default();
        if let Some(existing) = cm.msgs.iter_mut().find(|x| x.dto.id == m.dto.id) {
            if m.dto.media.is_some() && existing.dto.media.is_none() {
                existing.dto.media = m.dto.media.take();
                existing.dto.thumbnail = m.dto.thumbnail.take().or(existing.dto.thumbnail.take());
                self.dirty_msgs.insert(jid);
            }
            return false;
        }
        // Reconcile the server echo of our own send with its optimistic bubble.
        if m.dto.from_me
            && let Some(tmp) = cm.msgs.iter_mut().find(|x| {
                x.status == MessageStatus::Sending
                    && x.dto.id.starts_with("tmp-")
                    && x.dto.kind == m.dto.kind
                    && x.dto.text == m.dto.text
                    && x.dto.quoted.as_ref().map(|q| &q.id) == m.dto.quoted.as_ref().map(|q| &q.id)
            })
        {
            m.status = MessageStatus::Sent;
            *tmp = m;
            self.dirty_msgs.insert(jid);
            return true;
        }
        if m.dto.from_me && m.status == MessageStatus::None {
            m.status = MessageStatus::Sent;
        }
        if !m.dto.from_me {
            m.unread = true;
        }
        let preview = Self::preview_of(&m.dto);
        let (ts, incoming) = (m.dto.timestamp, !m.dto.from_me);
        let pos = cm.msgs.partition_point(|x| x.dto.timestamp <= ts);
        cm.msgs.insert(pos, m);
        self.dirty_msgs.insert(jid.clone());
        self.touch_chat(&jid, preview, ts, incoming);
        true
    }

    /// Bulk insert from history sync (dedupes by id).
    pub fn add_history(&mut self, list: Vec<UiMessage>) {
        let mut by_chat: HashMap<String, Vec<UiMessage>> = HashMap::new();
        for m in list {
            if !is_blank(&m.dto) {
                by_chat.entry(m.dto.chat_jid.clone()).or_default().push(m);
            }
        }
        for (jid, items) in by_chat {
            self.ensure_full(&jid);
            let cm = self.msgs.entry(jid.clone()).or_default();
            let mut seen: HashSet<String> = cm.msgs.iter().map(|m| m.dto.id.clone()).collect();
            for mut m in items {
                if !m.dto.id.is_empty() && !seen.insert(m.dto.id.clone()) {
                    continue;
                }
                if m.dto.from_me && m.status == MessageStatus::None {
                    m.status = MessageStatus::Sent;
                }
                cm.msgs.push(m);
            }
            cm.msgs.sort_by_key(|m| m.dto.timestamp);
            self.dirty_msgs.insert(jid.clone());
            self.apply_unread_flags(&jid);
        }
    }

    /// Find a message by id, paging the chat's full history in on a miss.
    fn message_mut(&mut self, jid: &str, id: &str) -> Option<&mut UiMessage> {
        let resident = self
            .msgs
            .get(jid)
            .is_some_and(|c| c.msgs.iter().any(|m| m.dto.id == id));
        if !resident {
            self.ensure_full(jid);
        }
        self.dirty_msgs.insert(jid.to_string());
        self.msgs
            .get_mut(jid)?
            .msgs
            .iter_mut()
            .find(|m| m.dto.id == id)
    }

    pub fn apply_receipt(&mut self, jid: &str, ids: &[String], status: MessageStatus) {
        for id in ids {
            match self
                .msgs
                .get_mut(jid)
                .and_then(|c| c.msgs.iter_mut().find(|m| &m.dto.id == id))
            {
                Some(m) => {
                    if m.dto.from_me && status.rank() > m.status.rank() {
                        m.status = status;
                        self.dirty_msgs.insert(jid.to_string());
                    }
                }
                None => {
                    let prev = self.early_receipts.get(id).copied().unwrap_or_default();
                    if status.rank() > prev.rank() {
                        self.early_receipts.insert(id.clone(), status);
                        if self.early_receipts.len() > 500 {
                            self.early_receipts.clear();
                        }
                    }
                }
            }
        }
    }

    /// Revoke from the wire: tombstone the message, or insert a tombstone when
    /// we never had it (so the "deleted" bubble still shows in place).
    pub fn apply_revoke_event(
        &mut self,
        jid: &str,
        id: &str,
        sender: &str,
        from_me: bool,
        ts: i64,
    ) {
        if self.message_mut(jid, id).is_none() && !id.is_empty() {
            let m = UiMessage {
                dto: MessageDto {
                    id: id.to_string(),
                    chat_jid: jid.to_string(),
                    sender_jid: sender.to_string(),
                    from_me,
                    timestamp: ts,
                    kind: "other".into(),
                    ..Default::default()
                },
                deleted: true,
                ..Default::default()
            };
            let cm = self.msgs.entry(jid.to_string()).or_default();
            let pos = cm.msgs.partition_point(|x| x.dto.timestamp <= ts);
            cm.msgs.insert(pos, m);
            self.dirty_msgs.insert(jid.to_string());
            return;
        }
        self.apply_revoke(jid, id);
    }

    pub fn apply_revoke(&mut self, jid: &str, id: &str) {
        if let Some(m) = self.message_mut(jid, id) {
            m.unread = false;
            m.deleted = true;
            m.dto.text = None;
            m.dto.media = None;
            m.dto.thumbnail = None;
            m.dto.quoted = None;
            m.reactions.clear();
        }
        self.refresh_preview(jid);
    }

    pub fn apply_edit(&mut self, jid: &str, id: &str, text: Option<String>, ts: i64) {
        if let Some(m) = self.message_mut(jid, id) {
            if text.is_some() {
                m.dto.text = text;
            }
            m.edited_at = Some(ts);
        }
        self.refresh_preview(jid);
    }

    pub fn apply_reaction(&mut self, jid: &str, id: &str, reactor: &str, emoji: &str) {
        if let Some(m) = self.message_mut(jid, id) {
            if emoji.is_empty() {
                m.reactions.remove(reactor);
            } else {
                m.reactions.insert(reactor.to_string(), emoji.to_string());
            }
        }
    }

    pub fn remove_message(&mut self, jid: &str, id: &str) {
        self.ensure_full(jid);
        if let Some(cm) = self.msgs.get_mut(jid) {
            cm.msgs.retain(|m| m.dto.id != id);
            self.dirty_msgs.insert(jid.to_string());
        }
        self.refresh_preview(jid);
    }

    fn refresh_preview(&mut self, jid: &str) {
        let last = self.messages(jid).last().map(|m| {
            if m.deleted {
                Some("🚫 This message was deleted".to_string())
            } else {
                Self::preview_of(&m.dto)
            }
        });
        if let (Some(preview), Some(c)) = (last, self.chats.get_mut(jid)) {
            c.last_message = preview;
            self.dirty_chats = true;
        }
    }

    /// Insert an optimistic outgoing bubble; returns its temporary id.
    pub fn add_optimistic(
        &mut self,
        jid: &str,
        kind: &str,
        text: Option<String>,
        thumbnail: Option<String>,
        quoted: Option<QuotedDto>,
    ) -> String {
        self.tmp_seq += 1;
        let id = format!("tmp-{}-{}", now_secs(), self.tmp_seq);
        let m = UiMessage {
            dto: MessageDto {
                id: id.clone(),
                chat_jid: jid.to_string(),
                from_me: true,
                timestamp: now_secs(),
                text,
                kind: kind.to_string(),
                thumbnail,
                quoted,
                ..Default::default()
            },
            status: MessageStatus::Sending,
            ..Default::default()
        };
        self.add_message(m);
        id
    }

    /// Swap an optimistic bubble's temp id for the server id.
    pub fn confirm_optimistic(&mut self, jid: &str, tmp: &str, real: &str) {
        let early = self
            .early_receipts
            .remove(real)
            .unwrap_or(MessageStatus::Sent);
        let Some(cm) = self.msgs.get_mut(jid) else {
            return;
        };
        if cm.msgs.iter().any(|m| m.dto.id == real) {
            // The server echo already arrived under the real id.
            cm.msgs.retain(|m| m.dto.id != tmp);
        } else if let Some(m) = cm.msgs.iter_mut().find(|m| m.dto.id == tmp) {
            m.dto.id = real.to_string();
            m.status = if early.rank() > MessageStatus::Sent.rank() {
                early
            } else {
                MessageStatus::Sent
            };
        }
        self.dirty_msgs.insert(jid.to_string());
    }

    pub fn fail_optimistic(&mut self, jid: &str, tmp: &str) {
        if let Some(m) = self
            .msgs
            .get_mut(jid)
            .and_then(|c| c.msgs.iter_mut().find(|m| m.dto.id == tmp))
        {
            m.status = MessageStatus::Failed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(chat: &str, id: &str, ts: i64, from_me: bool, text: &str) -> UiMessage {
        UiMessage {
            dto: MessageDto {
                id: id.into(),
                chat_jid: chat.into(),
                sender_jid: chat.into(),
                from_me,
                timestamp: ts,
                text: Some(text.into()),
                kind: "text".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn add_dedupes_and_bumps_unread() {
        let mut s = Store::new();
        assert!(s.add_message(msg("a@s.whatsapp.net", "1", 10, false, "hi")));
        assert!(!s.add_message(msg("a@s.whatsapp.net", "1", 10, false, "hi")));
        assert_eq!(s.chats["a@s.whatsapp.net"].unread, 1);
        assert_eq!(
            s.chats["a@s.whatsapp.net"].last_message.as_deref(),
            Some("hi")
        );
    }

    #[test]
    fn optimistic_is_confirmed_and_receipts_never_downgrade() {
        let mut s = Store::new();
        let jid = "a@s.whatsapp.net";
        let tmp = s.add_optimistic(jid, "text", Some("yo".into()), None, None);
        s.apply_receipt(jid, &["R".into()], MessageStatus::Read);
        s.confirm_optimistic(jid, &tmp, "R");
        assert_eq!(s.messages(jid)[0].status, MessageStatus::Read);
        s.apply_receipt(jid, &["R".into()], MessageStatus::Delivered);
        assert_eq!(s.messages(jid)[0].status, MessageStatus::Read);
    }

    #[test]
    fn learning_a_lid_merges_split_chats() {
        let mut s = Store::new();
        s.add_message(msg("1@s.whatsapp.net", "a", 1, false, "old"));
        s.add_message(msg("9@lid", "b", 2, false, "new"));
        s.learn_lid("9@lid", "1@s.whatsapp.net");
        assert!(!s.chats.contains_key("9@lid"));
        let c = &s.chats["1@s.whatsapp.net"];
        assert_eq!(c.unread, 2);
        assert_eq!(c.last_message.as_deref(), Some("new"));
        assert_eq!(s.messages("1@s.whatsapp.net").len(), 2);
        assert_eq!(s.canonical("9@lid"), "1@s.whatsapp.net");
    }

    #[test]
    fn unread_flags_and_mark_read() {
        let mut s = Store::new();
        let j = "a@s.whatsapp.net";
        s.add_message(msg(j, "1", 1, false, "a"));
        s.add_message(msg(j, "2", 2, false, "b"));
        s.open_chat(j);
        assert!(s.messages(j).iter().all(|m| m.unread));
        assert_eq!(s.mark_message_read(j, "1"), Some(false));
        assert_eq!(s.mark_message_read(j, "1"), None);
        assert_eq!(s.mark_message_read(j, "2"), Some(true));
        assert_eq!(s.chats[j].unread, 0);
    }

    #[test]
    fn revoke_of_unknown_message_inserts_tombstone() {
        let mut s = Store::new();
        s.apply_revoke_event("a@s.whatsapp.net", "x", "a@s.whatsapp.net", false, 5);
        assert!(s.messages("a@s.whatsapp.net")[0].deleted);
    }

    #[test]
    fn revoke_tombstones_message() {
        let mut s = Store::new();
        s.add_message(msg("a@s.whatsapp.net", "1", 1, false, "secret"));
        s.apply_revoke("a@s.whatsapp.net", "1");
        let m = &s.messages("a@s.whatsapp.net")[0];
        assert!(m.deleted && m.dto.text.is_none());
    }
}
