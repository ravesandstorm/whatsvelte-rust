//! The controller: owns the store, applies bridge events to it, runs user
//! actions against the API, and pushes coalesced view frames to Slint.
//!
//! Threading: everything here runs on the tokio runtime. State sits behind
//! `std::sync::Mutex`es that are never held across an `.await`. The UI thread
//! only receives finished [`Frame`]s (plain `Send` data + pixel buffers) and
//! copies them into the Slint models — it never decodes or computes.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tokio::sync::Notify;

use crate::core::api::Api;
use crate::core::bridge::UiEvent;
use crate::core::dto::{MessageDto, QuotedDto, StatusDto};
use crate::core::media::MediaSendOptionsIn;
use crate::core::paths;
use crate::store::persist::Persist;
use crate::store::settings::Settings;
use crate::store::{MessageStatus, Section, Store, UiMessage};
use crate::ui::frame::{ChatRowData, ChatsFrame, ConvFrame, Frame, MsgRowData, SessionFrame};
use crate::ui::images::{self, ImageCache, Pixels};
use crate::ui::qr;
use crate::util::jid::{format_phone, initials, is_group, is_lid, is_newsletter, is_phone, is_self, is_status, normalize_jid};
use crate::util::time::{clock, day_label, list_stamp, local_day, now_secs};
use crate::AppWindow;

const DIRTY_SESSION: u8 = 1;
const DIRTY_CHATS: u8 = 2;
const DIRTY_CONV: u8 = 4;
const DIRTY_ALL: u8 = 7;

/// Messages shown when a chat opens / added per "load older".
const PAGE: usize = 80;
/// Own text messages can be edited for this long (WhatsApp's window).
const EDIT_WINDOW_SECS: i64 = 15 * 60;

#[derive(Default)]
struct View {
    status: StatusDto,
    booted: bool,
    hydrating: bool,
    loading_label: String,
    loading_progress: f32,
    qr: Option<Pixels>,
    pair_code: String,
    pair_error: String,
    pair_busy: bool,
    dead: bool,
    section: Section,
    search: String,
    window_limit: usize,
    reply_to: Option<String>,
    editing: Option<String>,
    composer_override: Option<String>,
    last_history_chunk: Option<Instant>,
    syncing_shown: bool,
    newsletters_loaded: bool,
    toast: String,
    toast_until: Option<Instant>,
    scroll_token: i32,
    append_token: i32,
    viewer: Option<(String, Pixels)>,
}

pub struct Controller {
    api: Api,
    pub rt: tokio::runtime::Handle,
    window: slint::Weak<AppWindow>,
    store: Mutex<Store>,
    settings: Mutex<Settings>,
    view: Mutex<View>,
    images: Mutex<ImageCache>,
    /// Decrypted media already in the cache, by message id.
    media_paths: Mutex<HashMap<String, PathBuf>>,
    media_inflight: Mutex<HashSet<String>>,
    lid_inflight: Mutex<HashSet<String>>,
    dirty: AtomicU8,
    wake: Notify,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

/// Stable avatar / sender tint per jid.
pub fn tint_for(jid: &str) -> u32 {
    const PALETTE: [u32; 10] = [
        0x25d366, 0x34b7f1, 0xe542a3, 0xff8a65, 0x7e57c2, 0x26a69a, 0xf4511e, 0x5c6bc0, 0x8d6e63, 0x00897b,
    ];
    let h = jid.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    PALETTE[(h % PALETTE.len() as u32) as usize]
}

/// Avatar letters; unnamed numbers / LIDs get a person glyph instead of "+4".
fn avatar_initials(title: &str) -> String {
    if title.starts_with('+') || title.starts_with('~') || title.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        "👤".into()
    } else {
        initials(title)
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "image" => "📷 Photo",
        "video" => "🎥 Video",
        "audio" => "🎤 Audio",
        "document" => "📄 Document",
        "sticker" => "Sticker",
        "location" => "📍 Location",
        "contact" => "👤 Contact",
        "poll" => "📊 Poll",
        _ => "Message",
    }
}

fn fmt_duration(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn fmt_size(bytes: u64) -> String {
    match bytes {
        0 => String::new(),
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.0} KB", b as f64 / 1024.0),
        b => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
    }
}

impl Controller {
    pub fn new(api: Api, rt: tokio::runtime::Handle, window: slint::Weak<AppWindow>) -> Arc<Self> {
        let settings = Settings::load();
        Arc::new(Self {
            api,
            rt,
            window,
            store: Mutex::new(Store::new()),
            settings: Mutex::new(settings),
            view: Mutex::new(View {
                loading_label: "Connecting…".into(),
                loading_progress: -1.0,
                window_limit: PAGE,
                ..Default::default()
            }),
            images: Mutex::new(ImageCache::new(400)),
            media_paths: Mutex::new(HashMap::new()),
            media_inflight: Mutex::new(HashSet::new()),
            lid_inflight: Mutex::new(HashSet::new()),
            dirty: AtomicU8::new(DIRTY_ALL),
            wake: Notify::new(),
        })
    }

    fn mark(&self, bits: u8) {
        self.dirty.fetch_or(bits, Ordering::SeqCst);
        self.wake.notify_one();
    }

    fn toast(&self, text: impl Into<String>) {
        let mut v = lock(&self.view);
        v.toast = text.into();
        v.toast_until = Some(Instant::now() + Duration::from_secs(3));
        drop(v);
        self.mark(DIRTY_SESSION);
    }

    fn active(&self) -> Option<String> {
        lock(&self.store).active.clone()
    }

    fn self_jid(&self) -> Option<String> {
        lock(&self.view).status.jid.clone()
    }

    // ================================================================ loops

    /// Startup: boot the session, reconcile the on-disk cache against the
    /// handshake state, then start consuming events (port of startEventBridge).
    pub async fn run(self: Arc<Self>, events: async_channel::Receiver<UiEvent>) {
        tokio::spawn(self.clone().render_loop());
        tokio::spawn(self.clone().housekeeping_loop());

        if let Err(e) = self.api.auth_start_qr().await {
            log::error!("failed to boot default session: {e}");
        }
        self.refresh_status().await;

        let registered = lock(&self.view).status.registered;
        let persist = Persist::new(paths::ui_cache_dir());
        let cached_account = lock(&self.store).open_cache(persist);

        if !registered {
            // Logged out: any cache is stale (and a stale on-disk key can block
            // QR generation), so wipe both.
            if cached_account.is_some() {
                lock(&self.store).clear_all();
                if let Err(e) = self.api.reset_session().await {
                    log::error!("reset session failed: {e}");
                }
                self.refresh_status().await;
            }
        } else {
            {
                let mut v = lock(&self.view);
                v.hydrating = true;
                v.loading_label = "Restoring chats…".into();
            }
            self.mark(DIRTY_SESSION);
            let jid = self.self_jid();
            let different = matches!((&cached_account, &jid), (Some(a), Some(j)) if normalize_jid(a) != normalize_jid(j));
            let me = self.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let mut s = lock(&me.store);
                if different {
                    s.clear_all();
                } else {
                    s.hydrate();
                }
            })
            .await;
            let lid_jids: Vec<String> = lock(&self.store).chats.keys().filter(|j| is_lid(j)).cloned().collect();
            for j in lid_jids {
                tokio::spawn(self.clone().unify_lid(j));
            }
            lock(&self.view).hydrating = false;
        }
        self.learn_self_lid();
        if let Some(j) = self.self_jid() {
            lock(&self.store).set_account(&j);
        }
        lock(&self.view).booted = true;
        self.mark(DIRTY_ALL);
        self.clone().maybe_fetch_newsletters().await;

        while let Ok(ev) = events.recv().await {
            self.handle_event(ev).await;
        }
    }

    async fn render_loop(self: Arc<Self>) {
        loop {
            self.wake.notified().await;
            // Coalesce bursts (history sync delivers hundreds of chunks).
            tokio::time::sleep(Duration::from_millis(24)).await;
            let bits = self.dirty.swap(0, Ordering::SeqCst);
            if bits == 0 {
                continue;
            }
            let frame = self.build_frame(bits);
            if self.window.upgrade_in_event_loop(move |w| frame.apply(&w)).is_err() {
                break;
            }
        }
    }

    /// Periodic: persist dirty state, expire the toast and the syncing badge.
    async fn housekeeping_loop(self: Arc<Self>) {
        let mut tick = tokio::time::interval(Duration::from_millis(1000));
        loop {
            tick.tick().await;
            let me = self.clone();
            let _ = tokio::task::spawn_blocking(move || lock(&me.store).flush()).await;
            let mut v = lock(&self.view);
            let mut changed = false;
            if v.toast_until.is_some_and(|t| Instant::now() >= t) {
                v.toast.clear();
                v.toast_until = None;
                changed = true;
            }
            let syncing = v.last_history_chunk.is_some_and(|t| t.elapsed() < Duration::from_millis(2500));
            if syncing != v.syncing_shown {
                v.syncing_shown = syncing;
                changed = true;
            }
            drop(v);
            if changed {
                self.mark(DIRTY_SESSION | DIRTY_CHATS);
            }
        }
    }

    /// Final flush on window close.
    pub fn shutdown(&self) {
        lock(&self.store).flush();
    }

    // ================================================================ session

    async fn refresh_status(&self) {
        match self.api.auth_status().await {
            Ok(s) => {
                lock(&self.view).status = s;
                self.learn_self_lid();
            }
            Err(e) => log::error!("auth_status failed: {e}"),
        }
        self.mark(DIRTY_SESSION);
    }

    fn learn_self_lid(&self) {
        let (lid, jid) = {
            let v = lock(&self.view);
            (v.status.lid.clone(), v.status.jid.clone())
        };
        if let (Some(l), Some(j)) = (lid, jid) {
            lock(&self.store).learn_lid(&l, &j);
        }
    }

    async fn maybe_fetch_newsletters(self: Arc<Self>) {
        {
            let mut v = lock(&self.view);
            if v.newsletters_loaded || !v.status.connected {
                return;
            }
            v.newsletters_loaded = true;
        }
        match self.api.list_newsletters().await {
            Ok(rows) => {
                let mut s = lock(&self.store);
                for c in rows {
                    s.upsert_chat_dto(&c);
                    if let Some(n) = &c.name {
                        s.learn_name(&c.jid, n);
                    }
                }
                drop(s);
                self.mark(DIRTY_CHATS);
            }
            Err(e) => {
                lock(&self.view).newsletters_loaded = false;
                log::warn!("list newsletters failed: {e}");
            }
        }
    }

    /// Full reset: forget everything and regenerate the device key so a clean
    /// QR can be generated (logout / stuck session recovery).
    async fn reset_all(&self) {
        {
            let mut v = lock(&self.view);
            let booted = v.booted;
            *v = View {
                booted,
                loading_label: "Connecting…".into(),
                loading_progress: -1.0,
                window_limit: PAGE,
                ..Default::default()
            };
        }
        lock(&self.store).clear_all();
        lock(&self.media_paths).clear();
        self.mark(DIRTY_ALL);
        if let Err(e) = self.api.reset_session().await {
            log::error!("reset session failed: {e}");
        }
        self.refresh_status().await;
    }

    // ================================================================ events

    async fn handle_event(self: &Arc<Self>, ev: UiEvent) {
        match ev {
            UiEvent::Qr { code, .. } => {
                let px = qr::render(&code, 6);
                let mut v = lock(&self.view);
                v.dead = false;
                v.qr = px;
                drop(v);
                self.mark(DIRTY_SESSION);
            }
            UiEvent::PairCode { code, .. } => {
                lock(&self.view).pair_code = code;
                self.mark(DIRTY_SESSION);
            }
            UiEvent::Paired { .. } => {
                {
                    let mut v = lock(&self.view);
                    v.qr = None;
                    v.pair_code.clear();
                    v.pair_error.clear();
                }
                self.refresh_status().await;
                if let Some(j) = self.self_jid() {
                    lock(&self.store).set_account(&j);
                }
            }
            UiEvent::PairError { message, .. } => {
                lock(&self.view).pair_error = format!("Pairing failed: {message}");
                self.mark(DIRTY_SESSION);
            }
            UiEvent::LoggedOut { .. } => self.reset_all().await,
            UiEvent::ClientDead { .. } => {
                lock(&self.view).dead = true;
                self.mark(DIRTY_SESSION);
            }
            UiEvent::ConnState { .. } => {
                self.refresh_status().await;
                self.clone().maybe_fetch_newsletters().await;
            }
            UiEvent::Message { message, .. } => self.on_message(message),
            UiEvent::History { history, .. } => {
                lock(&self.view).last_history_chunk = Some(Instant::now());
                let mut lids = Vec::new();
                {
                    let mut s = lock(&self.store);
                    for c in &history.chats {
                        s.upsert_chat_dto(c);
                        if let Some(n) = &c.name {
                            let j = s.canonical(&c.jid);
                            s.learn_name(&j, n);
                        }
                        lids.push(c.jid.clone());
                    }
                    let mut list = Vec::with_capacity(history.messages.len());
                    for m in history.messages {
                        if m.sender_jid != m.chat_jid {
                            lids.push(m.sender_jid.clone());
                        }
                        let cm = s.canonical_message(m);
                        if let (Some(n), false) = (&cm.dto.push_name, cm.dto.from_me) {
                            let who = if cm.dto.sender_jid.is_empty() { cm.dto.chat_jid.clone() } else { cm.dto.sender_jid.clone() };
                            s.learn_name(&who, n);
                        }
                        list.push(cm);
                    }
                    s.add_history(list);
                }
                for j in lids.into_iter().filter(|j| is_lid(j)) {
                    tokio::spawn(self.clone().unify_lid(j));
                }
                self.mark(DIRTY_ALL);
            }
            UiEvent::Receipt { receipt, .. } => {
                let mut jid = lock(&self.store).canonical(&receipt.chat_jid);
                if is_lid(&jid) {
                    self.clone().unify_lid(receipt.chat_jid.clone()).await;
                    jid = lock(&self.store).canonical(&receipt.chat_jid);
                }
                lock(&self.store).apply_receipt(&jid, &receipt.message_ids, MessageStatus::parse(&receipt.status));
                self.mark_jid(&jid);
            }
            UiEvent::ChatFlags { flags, .. } => {
                lock(&self.store).apply_chat_flags(&flags);
                tokio::spawn(self.clone().unify_lid(flags.jid.clone()));
                self.mark(DIRTY_CHATS | DIRTY_CONV);
            }
            UiEvent::MessageUpdate { update, .. } => {
                let (jid, sender) = {
                    let s = lock(&self.store);
                    (s.canonical(&update.chat_jid), s.canonical(update.sender_jid.as_deref().unwrap_or("")))
                };
                {
                    let mut s = lock(&self.store);
                    match update.kind.as_str() {
                        "revoke" => s.apply_revoke(&jid, &update.target_id),
                        "edit" => s.apply_edit(&jid, &update.target_id, update.text.clone(), update.timestamp),
                        "reaction" => {
                            let reactor = if update.from_me {
                                self.self_jid().map(|j| normalize_jid(&j)).unwrap_or_else(|| normalize_jid(&sender))
                            } else {
                                normalize_jid(&sender)
                            };
                            s.apply_reaction(&jid, &update.target_id, &reactor, update.text.as_deref().unwrap_or(""));
                        }
                        _ => {}
                    }
                }
                tokio::spawn(self.clone().unify_lid(update.chat_jid.clone()));
                self.mark_jid(&jid);
            }
            UiEvent::DeleteForMe { chat_jid, message_id, .. } => {
                let jid = lock(&self.store).canonical(&chat_jid);
                lock(&self.store).remove_message(&jid, &message_id);
                tokio::spawn(self.clone().unify_lid(chat_jid));
                self.mark_jid(&jid);
            }
            UiEvent::StreamError { message, .. } => {
                log::warn!("stream error: {message}");
                self.toast(format!("Connection problem: {message}"));
            }
        }
    }

    fn mark_jid(&self, jid: &str) {
        let active = self.active().as_deref() == Some(jid);
        self.mark(DIRTY_CHATS | if active { DIRTY_CONV } else { 0 });
    }

    fn on_message(self: &Arc<Self>, m: MessageDto) {
        let raw_chat = m.chat_jid.clone();
        let raw_sender = m.sender_jid.clone();
        let (added, jid, from_me, id, wire_chat, wire_sender) = {
            let mut s = lock(&self.store);
            let cm = s.canonical_message(m);
            let jid = cm.dto.chat_jid.clone();
            if let (Some(n), false) = (cm.dto.push_name.clone(), cm.dto.from_me) {
                if !is_group(&jid) {
                    if let Some(c) = s.chats.get_mut(&jid)
                        && c.name.is_none()
                    {
                        c.name = Some(n.clone());
                    }
                }
                let who = if cm.dto.sender_jid.is_empty() { jid.clone() } else { cm.dto.sender_jid.clone() };
                s.learn_name(&who, &n);
            }
            let info = (cm.dto.from_me, cm.dto.id.clone(), cm.proto_chat_jid().to_string(), cm.proto_sender_jid().to_string());
            let added = s.add_message(cm);
            (added, jid, info.0, info.1, info.2, info.3)
        };

        let is_active = self.active().as_deref() == Some(jid.as_str());
        if added && is_active {
            lock(&self.view).append_token += 1;
            if !from_me && lock(&self.settings).send_read_receipts {
                let me = self.clone();
                let sender = is_group(&jid).then_some(wire_sender);
                tokio::spawn(async move {
                    if let Err(e) = me.api.mark_read_messages(wire_chat, sender, vec![id]).await {
                        log::warn!("read receipt failed: {e}");
                    }
                });
            }
        }
        tokio::spawn(self.clone().unify_lid(raw_chat.clone()));
        if raw_sender != raw_chat {
            tokio::spawn(self.clone().unify_lid(raw_sender));
        }
        self.mark(DIRTY_CHATS | if is_active { DIRTY_CONV } else { 0 });
    }

    /// Ensure a LID jid is unified with its PN form (port of lid.ts unifyLid).
    async fn unify_lid(self: Arc<Self>, jid: String) {
        if !is_lid(&jid) {
            return;
        }
        let jid = normalize_jid(&jid);
        let known = lock(&self.store).knows_lid(&jid);
        if known {
            let pn = lock(&self.store).canonical(&jid);
            lock(&self.store).learn_lid(&jid, &pn);
            return;
        }
        if !lock(&self.lid_inflight).insert(jid.clone()) {
            return;
        }
        match self.api.resolve_jid(jid.clone()).await {
            Ok(r) => {
                if let Some(pn) = r.pn.filter(|p| *p != jid) {
                    lock(&self.store).learn_lid(&jid, &pn);
                    self.mark(DIRTY_CHATS | DIRTY_CONV);
                }
            }
            Err(e) => log::debug!("lid resolve failed for {jid}: {e}"),
        }
        lock(&self.lid_inflight).remove(&jid);
    }

    // ================================================================ frames

    fn build_frame(&self, bits: u8) -> Frame {
        Frame {
            session: (bits & DIRTY_SESSION != 0).then(|| self.session_frame()),
            chats: (bits & DIRTY_CHATS != 0).then(|| self.chats_frame()),
            conv: (bits & DIRTY_CONV != 0).then(|| self.conv_frame()),
        }
    }

    fn session_frame(&self) -> SessionFrame {
        let settings = lock(&self.settings).clone();
        let mut v = lock(&self.view);
        let screen = if !v.booted || v.hydrating {
            "loading"
        } else if v.status.registered || v.status.logged_in {
            "main"
        } else {
            "pairing"
        };
        let viewer = v.viewer.take().map(Some);
        SessionFrame {
            screen: screen.into(),
            loading_label: v.loading_label.clone(),
            loading_progress: v.loading_progress,
            qr: v.qr.clone(),
            pair_code: v.pair_code.clone(),
            pair_error: v.pair_error.clone(),
            pair_busy: v.pair_busy,
            dead: v.dead,
            connected: v.status.connected,
            syncing: v.syncing_shown,
            self_name: v.status.push_name.clone().unwrap_or_default(),
            self_jid: v.status.jid.as_deref().map(format_phone).unwrap_or_default(),
            toast: v.toast.clone(),
            theme: settings.theme,
            zoom: settings.zoom,
            read_receipts: settings.send_read_receipts,
            auto_load_images: settings.auto_load_images,
            viewer,
        }
    }

    fn chats_frame(&self) -> ChatsFrame {
        let (section, search) = {
            let v = lock(&self.view);
            (v.section, v.search.clone())
        };
        let s = lock(&self.store);
        let active = s.active.clone();
        let rows = s
            .chat_list(section, &search)
            .into_iter()
            .map(|c| {
                let title = s.title_for(&c.jid);
                let draft = c.draft.as_deref().filter(|d| !d.trim().is_empty() && active.as_deref() != Some(&c.jid));
                ChatRowData {
                    jid: c.jid.clone(),
                    initials: avatar_initials(&title),
                    title,
                    preview: draft
                        .map(str::to_string)
                        .or_else(|| c.last_message.as_ref().map(|m| m.lines().next().unwrap_or("").to_string()))
                        .unwrap_or_default(),
                    stamp: list_stamp(c.timestamp),
                    unread: c.unread as i32,
                    muted: c.is_muted(),
                    pinned: c.is_pinned(),
                    archived: c.is_archived(),
                    draft: draft.is_some(),
                    tint: tint_for(&c.jid),
                    active: active.as_deref() == Some(&c.jid),
                }
            })
            .collect();
        let (archived, channels, status) = s.section_counts();
        ChatsFrame {
            rows,
            section: match section {
                Section::Chats => 0,
                Section::Archived => 1,
                Section::Channels => 2,
                Section::Status => 3,
            },
            archived_count: archived as i32,
            channels_count: channels as i32,
            status_count: status as i32,
        }
    }

    fn conv_frame(&self) -> ConvFrame {
        let (limit, reply_to, editing, composer, scroll_token, append_token) = {
            let mut v = lock(&self.view);
            (v.window_limit, v.reply_to.clone(), v.editing.clone(), v.composer_override.take(), v.scroll_token, v.append_token)
        };
        let self_jid = self.self_jid();
        let auto_load = lock(&self.settings).auto_load_images;
        let s = lock(&self.store);
        let Some(jid) = s.active.clone() else {
            return ConvFrame { composer, scroll_token, append_token, ..Default::default() };
        };
        let chat = s.chats.get(&jid);
        let title = s.title_for(&jid);
        let group = is_group(&jid);
        let subtitle = if group {
            "Group".to_string()
        } else if is_newsletter(&jid) {
            "Channel".to_string()
        } else if is_status(&jid) {
            "Status updates".to_string()
        } else if is_phone(&jid) && format_phone(&jid) != title {
            format_phone(&jid)
        } else {
            String::new()
        };

        let all = s.messages(&jid);
        let start = all.len().saturating_sub(limit);
        let shown = &all[start..];
        let mut to_load: Vec<UiMessage> = Vec::new();
        let inflight = lock(&self.media_inflight).clone();
        let mut img_cache = lock(&self.images);
        let mut rows = Vec::with_capacity(shown.len());
        let mut prev_day: Option<i64> = None;
        for m in shown {
            let d = &m.dto;
            let day = local_day(d.timestamp);
            let day_text = if prev_day != Some(day) { day_label(d.timestamp) } else { String::new() };
            prev_day = Some(day);

            let sender_name = if d.from_me {
                "You".to_string()
            } else {
                let sj = if d.sender_jid.is_empty() { &jid } else { &d.sender_jid };
                let t = s.title_for(&normalize_jid(sj));
                if t.starts_with('~') || t.starts_with('+') {
                    d.push_name.clone().filter(|p| !p.is_empty()).map(|p| format!("{t} ~{p}")).unwrap_or(t)
                } else {
                    t
                }
            };

            // Image: full-size decode when cached, else the inline thumbnail.
            let full_key = format!("full:{}", d.id);
            let thumb_key = format!("thumb:{}", d.id);
            let wants_full = matches!(d.kind.as_str(), "image" | "sticker") && d.media.is_some() && !m.deleted;
            let mut image = img_cache.get(&full_key);
            if image.is_none() {
                image = img_cache.get(&thumb_key);
                if image.is_none()
                    && let Some(px) = d.thumbnail.as_deref().filter(|_| !m.deleted).and_then(images::decode_b64)
                {
                    img_cache.insert(thumb_key, px.clone());
                    image = Some(px);
                }
                if wants_full && auto_load && !inflight.contains(&d.id) {
                    to_load.push(m.clone());
                }
            }

            let media_label = match (&d.media, d.kind.as_str()) {
                (Some(md), "video") => format!("🎥 Video{}", md.duration_secs.map(|s| format!(" · {}", fmt_duration(s))).unwrap_or_default()),
                (Some(md), "audio") => format!("🎤 Audio{}", md.duration_secs.map(|s| format!(" · {}", fmt_duration(s))).unwrap_or_default()),
                (Some(md), "document") => {
                    let name = md.file_name.clone().unwrap_or_else(|| "Document".into());
                    let size = fmt_size(md.descriptor.file_length);
                    if size.is_empty() { format!("📄 {name}") } else { format!("📄 {name} · {size}") }
                }
                (Some(_), k) => kind_label(k).to_string(),
                (None, _) => String::new(),
            };

            let (has_quote, quote_sender, quote_text) = match &d.quoted {
                Some(q) => {
                    let qs = match q.sender_jid.as_deref() {
                        Some(sj) if is_self(sj, self_jid.as_deref()) || is_self(&s.canonical(&normalize_jid(sj)), self_jid.as_deref()) => "You".to_string(),
                        Some(sj) => s.title_for(&s.canonical(&normalize_jid(sj))),
                        None if d.from_me => title.clone(),
                        None => "You".to_string(),
                    };
                    let qt = q.text.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| kind_label(&q.kind).to_string());
                    (true, qs, qt)
                }
                None => (false, String::new(), String::new()),
            };

            let mut counts: Vec<(String, usize)> = Vec::new();
            for e in m.reactions.values() {
                match counts.iter_mut().find(|(x, _)| x == e) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((e.clone(), 1)),
                }
            }
            let reactions = counts
                .iter()
                .map(|(e, n)| if *n > 1 { format!("{e} {n}") } else { e.clone() })
                .collect::<Vec<_>>()
                .join(" ");

            let pending = m.is_pending();
            let (iw, ih) = image.as_ref().map(|p| (p.width() as i32, p.height() as i32)).unwrap_or_else(|| {
                d.media.as_ref().map(|md| (md.width.unwrap_or(0) as i32, md.height.unwrap_or(0) as i32)).unwrap_or((0, 0))
            });
            rows.push(MsgRowData {
                id: d.id.clone(),
                from_me: d.from_me,
                show_sender: group && !d.from_me,
                tint: tint_for(&d.sender_jid),
                sender: sender_name,
                text: if m.deleted { String::new() } else { d.text.clone().unwrap_or_default() },
                kind: d.kind.clone(),
                time: clock(d.timestamp),
                status: match m.status {
                    MessageStatus::None => 0,
                    MessageStatus::Failed => 1,
                    MessageStatus::Sending => 2,
                    MessageStatus::Sent => 3,
                    MessageStatus::Delivered => 4,
                    MessageStatus::Read => 5,
                    MessageStatus::Played => 6,
                },
                edited: m.edited_at.is_some(),
                deleted: m.deleted,
                reactions,
                has_quote,
                quote_sender,
                quote_text,
                image_w: iw,
                image_h: ih,
                image,
                has_media: d.media.is_some() && !m.deleted,
                media_label,
                loading_media: inflight.contains(&d.id),
                day: day_text,
                pending,
                can_edit: d.from_me && !m.deleted && !pending && d.kind == "text" && now_secs() - d.timestamp < EDIT_WINDOW_SECS,
                can_revoke: d.from_me && !m.deleted && !pending,
            });
        }
        drop(img_cache);

        let quote_target = reply_to.as_deref().or(editing.as_deref()).and_then(|id| s.find_message(&jid, id));
        let (reply_sender, reply_text) = quote_target
            .map(|q| {
                let who = if q.dto.from_me { "You".to_string() } else { s.title_for(&normalize_jid(if q.dto.sender_jid.is_empty() { &jid } else { &q.dto.sender_jid })) };
                (who, q.dto.text.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| kind_label(&q.dto.kind).to_string()))
            })
            .unwrap_or_default();

        let frame = ConvFrame {
            active_jid: jid.clone(),
            initials: avatar_initials(&title),
            title,
            subtitle,
            tint: tint_for(&jid),
            muted: chat.is_some_and(|c| c.is_muted()),
            pinned: chat.is_some_and(|c| c.is_pinned()),
            archived: chat.is_some_and(|c| c.is_archived()),
            rows,
            has_older: all.len() > limit,
            replying: reply_to.is_some(),
            editing: editing.is_some(),
            reply_sender,
            reply_text,
            composer,
            scroll_token,
            append_token,
        };
        drop(s);
        for m in to_load {
            self.spawn_image_load(m);
        }
        frame
    }

    // ================================================================ media

    fn spawn_image_load(&self, m: UiMessage) {
        if !lock(&self.media_inflight).insert(m.dto.id.clone()) {
            return;
        }
        // `self` is behind an Arc owned by the caller's tasks; re-acquire it via
        // the runtime handle by cloning the pieces we need.
        let api = self.api.clone();
        let id = m.dto.id.clone();
        let ctrl = CTRL.get().cloned();
        self.rt.spawn(async move {
            let Some(ctrl) = ctrl else { return };
            let path = ctrl.media_path(&api, &m).await;
            if let Some(path) = path {
                let px = tokio::task::spawn_blocking(move || images::decode_file(&path, images::BUBBLE_MAX_EDGE))
                    .await
                    .ok()
                    .flatten();
                if let Some(px) = px {
                    lock(&ctrl.images).insert(format!("full:{id}"), px);
                }
            }
            lock(&ctrl.media_inflight).remove(&id);
            ctrl.mark(DIRTY_CONV);
        });
    }

    /// Download (or hit the cache for) a message's media; returns its path.
    async fn media_path(&self, api: &Api, m: &UiMessage) -> Option<PathBuf> {
        if let Some(p) = lock(&self.media_paths).get(&m.dto.id).filter(|p| p.exists()) {
            return Some(p.clone());
        }
        let md = m.dto.media.as_ref()?;
        match api.download_media(md.descriptor.clone(), md.mimetype.clone(), md.file_name.clone()).await {
            Ok(p) => {
                let p = PathBuf::from(p);
                lock(&self.media_paths).insert(m.dto.id.clone(), p.clone());
                Some(p)
            }
            Err(e) => {
                log::warn!("media download failed for {}: {e}", m.dto.id);
                self.toast("Couldn't download media");
                None
            }
        }
    }

    fn active_message(&self, id: &str) -> Option<UiMessage> {
        let s = lock(&self.store);
        let jid = s.active.clone()?;
        s.find_message(&jid, id).cloned()
    }

    // ================================================================ actions

    pub async fn select_chat(self: Arc<Self>, jid: String) {
        let me = self.clone();
        let j = jid.clone();
        let changed_unread = tokio::task::spawn_blocking(move || {
            let mut s = lock(&me.store);
            s.open_chat(&j);
            s.mark_chat_read(&j)
        })
        .await
        .unwrap_or(false);
        let draft = lock(&self.store).chats.get(&jid).and_then(|c| c.draft.clone()).unwrap_or_default();
        {
            let mut v = lock(&self.view);
            v.window_limit = PAGE;
            v.reply_to = None;
            v.editing = None;
            v.composer_override = Some(draft);
            v.scroll_token += 1;
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        if changed_unread && lock(&self.settings).send_read_receipts {
            let proto = lock(&self.store).messages(&jid).iter().rev().find_map(|m| m.wire_chat_jid.clone()).unwrap_or(jid.clone());
            if let Err(e) = self.api.mark_read(proto).await {
                log::warn!("mark read failed: {e}");
            }
        }
    }

    pub fn set_section(&self, section: i32) {
        lock(&self.view).section = match section {
            1 => Section::Archived,
            2 => Section::Channels,
            3 => Section::Status,
            _ => Section::Chats,
        };
        self.mark(DIRTY_CHATS);
    }

    pub fn set_search(&self, q: String) {
        lock(&self.view).search = q;
        self.mark(DIRTY_CHATS);
    }

    pub async fn new_chat(self: Arc<Self>, input: String) {
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 6 {
            self.toast("Enter a phone number with country code");
            return;
        }
        lock(&self.view).section = Section::Chats;
        self.select_chat(format!("{digits}@s.whatsapp.net")).await;
    }

    pub async fn toggle_flag(self: Arc<Self>, flag: &'static str) {
        let Some(jid) = self.active() else { return };
        let current = {
            let s = lock(&self.store);
            let Some(c) = s.chats.get(&jid) else { return };
            match flag {
                "muted" => c.is_muted(),
                "pinned" => c.is_pinned(),
                _ => c.is_archived(),
            }
        };
        let value = !current;
        lock(&self.store).set_flag(&jid, flag, value);
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        let result = match flag {
            "muted" => self.api.set_chat_muted(jid.clone(), value).await,
            "pinned" => self.api.set_chat_pinned(jid.clone(), value).await,
            _ => self.api.set_chat_archived(jid.clone(), value).await,
        };
        if let Err(e) = result {
            lock(&self.store).set_flag(&jid, flag, current);
            self.mark(DIRTY_CHATS | DIRTY_CONV);
            self.toast(format!("Couldn't update chat: {e}"));
        }
    }

    pub fn load_older(&self) {
        lock(&self.view).window_limit += PAGE;
        self.mark(DIRTY_CONV);
    }

    pub fn composer_edited(&self, text: String) {
        if let Some(jid) = self.active() {
            lock(&self.store).set_draft(&jid, &text);
        }
    }

    pub fn cancel_compose(&self) {
        let mut v = lock(&self.view);
        if v.editing.take().is_some() {
            v.composer_override = Some(String::new());
        }
        v.reply_to = None;
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub fn reply_to(&self, id: String) {
        let mut v = lock(&self.view);
        v.editing = None;
        v.reply_to = Some(id);
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub fn start_edit(&self, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        let mut v = lock(&self.view);
        v.reply_to = None;
        v.editing = Some(id);
        v.composer_override = Some(m.dto.text.unwrap_or_default());
        drop(v);
        self.mark(DIRTY_CONV);
    }

    pub async fn send(self: Arc<Self>, text: String) {
        let text = text.trim_end().to_string();
        let Some(jid) = self.active() else { return };
        if text.trim().is_empty() {
            return;
        }
        let (reply_to, editing) = {
            let mut v = lock(&self.view);
            v.composer_override = Some(String::new());
            v.scroll_token += 1;
            (v.reply_to.take(), v.editing.take())
        };
        lock(&self.store).set_draft(&jid, "");

        if let Some(id) = editing {
            let Some(m) = self.active_message(&id) else { return };
            lock(&self.store).apply_edit(&jid, &id, Some(text.clone()), now_secs());
            self.mark(DIRTY_CHATS | DIRTY_CONV);
            if let Err(e) = self.api.edit_message(m.proto_chat_jid().to_string(), id, text).await {
                self.toast(format!("Edit failed: {e}"));
            }
            return;
        }

        let quoted_msg = reply_to.and_then(|id| self.active_message(&id));
        let quoted = quoted_msg.as_ref().map(|q| QuotedDto {
            id: q.dto.id.clone(),
            sender_jid: Some(if q.dto.from_me { self.self_jid().unwrap_or_default() } else { q.dto.sender_jid.clone() }),
            text: q.dto.text.clone(),
            kind: q.dto.kind.clone(),
            thumbnail: q.dto.thumbnail.clone(),
        });
        let tmp = lock(&self.store).add_optimistic(&jid, "text", Some(text.clone()), None, quoted);
        self.mark(DIRTY_CHATS | DIRTY_CONV);

        let proto_chat = jid.clone();
        let result = match &quoted_msg {
            Some(q) => {
                let sender = if q.dto.from_me {
                    self.self_jid().map(|j| normalize_jid(&j)).unwrap_or_default()
                } else if q.proto_sender_jid().is_empty() {
                    q.proto_chat_jid().to_string()
                } else {
                    q.proto_sender_jid().to_string()
                };
                self.api.send_reply(proto_chat, text, q.dto.id.clone(), sender, q.dto.text.clone()).await
            }
            None => self.api.send_text(proto_chat, text).await,
        };
        self.finish_send(&jid, &tmp, result.map(|r| r.message_id));
    }

    fn finish_send(&self, jid: &str, tmp: &str, result: Result<String, crate::core::error::ApiError>) {
        match result {
            Ok(real) => lock(&self.store).confirm_optimistic(jid, tmp, &real),
            Err(e) => {
                lock(&self.store).fail_optimistic(jid, tmp);
                self.toast(format!("Send failed: {e}"));
            }
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
    }

    pub async fn retry(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        if m.status != MessageStatus::Failed || m.dto.kind != "text" {
            return;
        }
        lock(&self.store).remove_message(&m.dto.chat_jid, &id);
        self.send(m.dto.text.unwrap_or_default()).await;
    }

    pub async fn react(self: Arc<Self>, id: String, emoji: String) {
        let Some(m) = self.active_message(&id) else { return };
        if m.is_pending() || m.deleted {
            return;
        }
        let jid = m.dto.chat_jid.clone();
        let me_jid = self.self_jid().map(|j| normalize_jid(&j)).unwrap_or_default();
        lock(&self.store).apply_reaction(&jid, &id, &me_jid, &emoji);
        self.mark(DIRTY_CONV);
        let participant = (is_group(&jid) && !m.dto.from_me).then(|| m.proto_sender_jid().to_string());
        if let Err(e) = self.api.send_reaction(m.proto_chat_jid().to_string(), id, m.dto.from_me, emoji, participant).await {
            self.toast(format!("Reaction failed: {e}"));
        }
    }

    pub async fn delete_for_me(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        let jid = m.dto.chat_jid.clone();
        lock(&self.store).remove_message(&jid, &id);
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        if m.is_pending() {
            return;
        }
        let participant = (is_group(&jid) && !m.dto.from_me).then(|| m.proto_sender_jid().to_string());
        if let Err(e) = self
            .api
            .delete_for_me(m.proto_chat_jid().to_string(), id, m.dto.from_me, participant, Some(m.dto.timestamp))
            .await
        {
            self.toast(format!("Delete failed: {e}"));
        }
    }

    pub async fn delete_for_everyone(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        if !m.dto.from_me || m.is_pending() {
            return;
        }
        let jid = m.dto.chat_jid.clone();
        match self.api.revoke_message(m.proto_chat_jid().to_string(), id.clone(), None).await {
            Ok(()) => {
                lock(&self.store).apply_revoke(&jid, &id);
                self.mark(DIRTY_CHATS | DIRTY_CONV);
            }
            Err(e) => self.toast(format!("Delete failed: {e}")),
        }
    }

    pub async fn open_media(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        if m.dto.media.is_none() {
            return;
        }
        lock(&self.media_inflight).insert(id.clone());
        self.mark(DIRTY_CONV);
        let path = self.media_path(&self.api.clone(), &m).await;
        lock(&self.media_inflight).remove(&id);
        self.mark(DIRTY_CONV);
        if let Some(p) = path
            && let Err(e) = open::that_detached(&p)
        {
            self.toast(format!("Couldn't open file: {e}"));
        }
    }

    pub async fn save_media(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        let Some(md) = m.dto.media.clone() else { return };
        self.toast("Saving…");
        match self.api.save_media_to_downloads(md.descriptor, md.mimetype, md.file_name).await {
            Ok(p) => {
                let name = Path::new(&p).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(p);
                self.toast(format!("Saved to Downloads: {name}"));
            }
            Err(e) => self.toast(format!("Save failed: {e}")),
        }
    }

    pub async fn view_image(self: Arc<Self>, id: String) {
        let Some(m) = self.active_message(&id) else { return };
        if m.dto.kind == "video" {
            return self.open_media(id).await;
        }
        let path = self.media_path(&self.api.clone(), &m).await;
        let px = match path {
            Some(p) => tokio::task::spawn_blocking(move || images::decode_file(&p, 2048)).await.ok().flatten(),
            None => None,
        }
        .or_else(|| lock(&self.images).get(&format!("thumb:{id}")));
        if let Some(px) = px {
            lock(&self.view).viewer = Some((id, px));
            self.mark(DIRTY_SESSION);
        }
    }

    pub async fn attach(self: Arc<Self>) {
        let Some(jid) = self.active() else { return };
        let Some(file) = rfd::AsyncFileDialog::new().set_title("Send a file").pick_file().await else {
            return;
        };
        let path = file.path().to_path_buf();
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let (media_type, mimetype) = match ext.as_str() {
            "jpg" | "jpeg" => ("image", "image/jpeg"),
            "png" => ("image", "image/png"),
            "webp" => ("image", "image/webp"),
            "mp4" | "m4v" => ("video", "video/mp4"),
            "mov" => ("video", "video/quicktime"),
            "mp3" => ("audio", "audio/mpeg"),
            "m4a" | "aac" => ("audio", "audio/mp4"),
            "ogg" | "opus" => ("audio", "audio/ogg; codecs=opus"),
            "pdf" => ("document", "application/pdf"),
            "txt" => ("document", "text/plain"),
            "zip" => ("document", "application/zip"),
            "docx" => ("document", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
            "xlsx" => ("document", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            "pptx" => ("document", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
            _ => ("document", "application/octet-stream"),
        };
        let thumb = if media_type == "image" {
            let p = path.clone();
            tokio::task::spawn_blocking(move || images::jpeg_thumbnail(&p)).await.ok().flatten()
        } else {
            None
        };
        let label = match media_type {
            "image" => None,
            "video" => Some(format!("🎥 {name}")),
            "audio" => Some(format!("🎵 {name}")),
            _ => Some(format!("📄 {name}")),
        };
        let tmp = lock(&self.store).add_optimistic(&jid, media_type, label, thumb.clone(), None);
        {
            let mut v = lock(&self.view);
            v.scroll_token += 1;
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        let opts = MediaSendOptionsIn {
            mimetype: Some(mimetype.to_string()),
            file_name: (media_type == "document").then(|| name.clone()),
            jpeg_thumbnail: thumb,
            ..Default::default()
        };
        let result = self
            .api
            .send_media(jid.clone(), path.to_string_lossy().into_owned(), media_type.to_string(), Some(opts))
            .await;
        self.finish_send(&jid, &tmp, result.map(|r| r.message_id));
    }

    pub async fn request_pair_code(self: Arc<Self>, phone: String) {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 6 {
            lock(&self.view).pair_error = "Enter your full phone number with country code.".into();
            self.mark(DIRTY_SESSION);
            return;
        }
        {
            let mut v = lock(&self.view);
            v.pair_busy = true;
            v.pair_error.clear();
        }
        self.mark(DIRTY_SESSION);
        let result = self.api.auth_start_pair_code(digits, None).await;
        let mut v = lock(&self.view);
        v.pair_busy = false;
        match result {
            Ok(c) => v.pair_code = c.code,
            Err(e) => v.pair_error = format!("Couldn't get a pairing code: {e}"),
        }
        drop(v);
        self.mark(DIRTY_SESSION);
    }

    pub async fn logout(self: Arc<Self>) {
        if let Err(e) = self.api.auth_logout().await {
            log::warn!("logout failed: {e}");
        }
        self.reset_all().await;
    }

    pub async fn reset_session(self: Arc<Self>) {
        self.reset_all().await;
    }

    pub fn update_settings(&self, f: impl FnOnce(&mut Settings)) {
        let mut s = lock(&self.settings);
        f(&mut s);
        s.save();
        drop(s);
        self.mark(DIRTY_ALL);
    }
}

/// The controller, reachable from detached tasks spawned by `&self` methods.
pub static CTRL: std::sync::OnceLock<Arc<Controller>> = std::sync::OnceLock::new();

/// Debug-only: fill the UI with sample chats (no session boot, no network) so
/// layouts can be checked without linking a real account. `WA_DEMO=1`.
#[cfg(debug_assertions)]
impl Controller {
    pub async fn run_demo(self: Arc<Self>) {
        use crate::core::dto::{ChatDto, MediaDescriptorDto, MediaDto};
        tokio::spawn(self.clone().render_loop());
        let now = now_secs();
        let me = "15550000000@s.whatsapp.net";
        {
            let mut v = lock(&self.view);
            v.booted = true;
            v.status = StatusDto {
                session_id: "default".into(),
                logged_in: true,
                connected: true,
                jid: Some(me.into()),
                registered: true,
                push_name: Some("Demo User".into()),
                ..Default::default()
            };
        }
        let chats = [
            ("919876543210@s.whatsapp.net", Some("Ada Lovelace"), "See you at 6!", 60, 2),
            ("120363000000000001@g.us", Some("Rust Hackers"), "cargo build --release 🚀", 600, 14),
            ("447700900123@s.whatsapp.net", None, "📷 Photo", 4000, 0),
            ("100000012345678@lid", None, "ok", 90_000, 0),
            ("120363000000000002@newsletter", Some("Slint News"), "1.18 released", 200_000, 0),
        ];
        {
            let mut s = lock(&self.store);
            for (jid, name, last, ago, unread) in chats {
                s.upsert_chat_dto(&ChatDto {
                    jid: jid.into(),
                    name: name.map(Into::into),
                    last_message: Some(last.into()),
                    timestamp: now - ago,
                    unread,
                });
            }
            s.set_flag("447700900123@s.whatsapp.net", "pinned", true);
            s.set_flag("120363000000000001@g.us", "muted", true);
            let g = "120363000000000001@g.us";
            let mk = |id: &str, sender: &str, from_me: bool, ago: i64, text: Option<&str>, kind: &str| {
                UiMessage {
                    dto: MessageDto {
                        id: id.into(),
                        chat_jid: g.into(),
                        sender_jid: sender.into(),
                        from_me,
                        timestamp: now - ago,
                        text: text.map(Into::into),
                        kind: kind.into(),
                        ..Default::default()
                    },
                    status: if from_me { MessageStatus::Read } else { MessageStatus::None },
                    ..Default::default()
                }
            };
            let mut msgs = vec![
                mk("m1", "919876543210@s.whatsapp.net", false, 90_000, Some("Has anyone tried Slint for desktop apps?"), "text"),
                mk("m2", me, true, 89_000, Some("Yes! Rewriting our Tauri client in it right now. No WebView at all."), "text"),
                mk("m3", "447700900123@s.whatsapp.net", false, 3000, Some("How's the memory usage?"), "text"),
            ];
            let mut reply = mk("m4", me, true, 2000, Some("Way lower — no JS heap, no IPC serialization."), "text");
            reply.dto.quoted = Some(QuotedDto {
                id: "m3".into(),
                sender_jid: Some("447700900123@s.whatsapp.net".into()),
                text: Some("How's the memory usage?".into()),
                kind: "text".into(),
                thumbnail: None,
            });
            reply.reactions.insert("a".into(), "🔥".into());
            reply.reactions.insert("b".into(), "🔥".into());
            reply.reactions.insert("c".into(), "👍".into());
            msgs.push(reply);
            let mut doc = mk("m5", "919876543210@s.whatsapp.net", false, 1500, None, "document");
            doc.dto.media = Some(MediaDto {
                kind: "document".into(),
                file_name: Some("benchmark-results.pdf".into()),
                descriptor: MediaDescriptorDto { file_length: 482_133, ..Default::default() },
                ..Default::default()
            });
            msgs.push(doc);
            let mut edited = mk("m6", me, true, 600, Some("cargo build --release 🚀"), "text");
            edited.edited_at = Some(now - 500);
            edited.status = MessageStatus::Delivered;
            msgs.push(edited);
            let mut deleted = mk("m7", "447700900123@s.whatsapp.net", false, 300, None, "text");
            deleted.deleted = true;
            msgs.push(deleted);
            s.add_history(msgs);
        }
        self.clone().select_chat("120363000000000001@g.us".into()).await;
        self.mark(DIRTY_ALL);
    }
}
