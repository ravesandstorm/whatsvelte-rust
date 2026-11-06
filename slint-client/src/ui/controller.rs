//! The controller: owns the store, applies bridge events to it, runs user
//! actions against the API, and pushes coalesced view frames to Slint.
//!
//! Threading: everything here runs on the tokio runtime. State sits behind
//! `std::sync::Mutex`es that are never held across an `.await`. The UI thread
//! only receives finished frames (plain `Send` data + pixel buffers).
//!
//! Split across files: this one (state, loops, boot, events, receipts),
//! `views.rs` (frame building), `actions.rs` (chat/message actions) and
//! `attach.rs` (media, attachments, GIFs, recording, avatars, lightbox).

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use tokio::sync::{Notify, Semaphore};

use crate::AppWindow;
use crate::core::api::Api;
use crate::core::bridge::UiEvent;
use crate::core::dto::{MessageDto, StatusDto};
use crate::core::paths;
use crate::media::audio::AudioPlayer;
use crate::media::camera::CameraSession;
use crate::media::klipy::Gif;
use crate::media::voice::VoiceRecorder;
use crate::store::persist::Persist;
use crate::store::settings::Settings;
use crate::store::{MessageStatus, Section, Store};
use crate::ui::images::{ImageCache, Pixels};
use crate::ui::qr;
use crate::util::jid::{is_group, is_lid, normalize_jid};

pub(crate) const DIRTY_SESSION: u8 = 1;
pub(crate) const DIRTY_CHATS: u8 = 2;
pub(crate) const DIRTY_CONV: u8 = 4;
pub(crate) const DIRTY_OVERLAY: u8 = 8;
pub(crate) const DIRTY_ALL: u8 = 15;

/// Rows kept built on each side of the row the list last asked for. The model
/// reports the chat's full length, so the list can scroll anywhere; only this
/// window holds real row data (and references images).
pub(crate) const ROW_WINDOW: usize = 60;
/// Rebuild once the list is this close to the window's edge.
const WINDOW_MARGIN: usize = 30;
/// Rows around the viewport whose full image may load; the rest show thumbnails.
pub(crate) const NEAR_ROWS: usize = 12;
/// Rebuild after this many rows of scrolling so nearby full images get requested.
const NEAR_STEP: usize = 8;
/// `focus` value meaning "the newest row".
pub(crate) const FOCUS_TAIL: usize = usize::MAX;
/// Decoded in-bubble images kept in memory. Evicted ones reload from disk.
const IMAGE_CACHE_BYTES: usize = 40 * 1024 * 1024;
/// Media downloads + decodes running at once. A full-size decode can briefly
/// take tens of megabytes, so this bounds the peak as well as the thread count.
const MEDIA_LOADS: usize = 3;
/// Toasts: lifetime and how many show at once.
const TOAST_TTL: Duration = Duration::from_millis(5000);
const TOAST_MAX: usize = 3;
/// Read receipts are batched for this long after a message becomes visible.
const RECEIPT_DEBOUNCE: Duration = Duration::from_millis(600);
const ACKED_MAX: usize = 5000;

#[derive(Default)]
pub(crate) struct GifState {
    pub query: String,
    pub items: Vec<Gif>,
    pub previews: HashMap<String, Pixels>,
    pub page: u32,
    pub has_next: bool,
    pub loading: bool,
    pub status: String,
    /// Bumped per new query so stale responses / debounces are dropped.
    pub seq: u64,
}

pub(crate) struct Preview {
    pub kind: String,
    pub path: PathBuf,
    pub name: String,
    pub image: Option<Pixels>,
    pub duration: Option<u32>,
    /// Temp file we created (recording/paste/capture) — delete after send.
    pub staged: bool,
}

pub(crate) enum Rec {
    Voice(VoiceRecorder),
    Photo {
        session: CameraSession,
        frame: Option<Pixels>,
        error: Option<String>,
    },
}

pub(crate) struct Viewer {
    pub ids: Vec<String>,
    pub index: usize,
    pub image: Option<Pixels>,
    pub loading: bool,
    pub error: String,
    pub is_video: bool,
    pub save_label: String,
    pub generation: u64,
}

#[derive(Default)]
pub(crate) struct View {
    pub status: StatusDto,
    pub booted: bool,
    pub hydrating: bool,
    pub loading_label: String,
    pub loading_done: i32,
    pub loading_total: i32,
    pub resetting: bool,
    pub qr: Option<Pixels>,
    pub pair_code: String,
    pub pair_error: String,
    pub pair_busy: bool,
    pub dead: bool,
    pub section: Section,
    pub search: String,
    /// (chat, id at the window's first row, its row index, total rows): detects
    /// history inserted above the viewport.
    pub conv_anchor: Option<(String, String, usize, usize)>,
    /// Message ids near the viewport; queued image loads for others are dropped.
    pub near_ids: HashSet<String>,
    /// First unread message when the chat opened. The "Unread" divider stays
    /// there while reading, rather than running ahead of the rows being read.
    pub unread_divider: Option<String>,
    pub reply_to: Option<String>,
    pub editing: Option<String>,
    /// Text that was in the composer when editing started (restored after).
    pub edit_stash: Option<String>,
    pub composer_text: String,
    pub composer_override: Option<(String, Option<i32>)>,
    pub focus_token: i32,
    pub suggestions: Vec<(String, String)>,
    pub suggestion_index: i32,
    pub suggestion_start: usize,
    pub suggestion_caret: usize,
    pub last_history_chunk: Option<Instant>,
    pub syncing_shown: bool,
    pub newsletters_loaded: bool,
    pub toasts: Vec<(String, Instant)>,
    pub scroll_token: i32,
    pub reveal: Option<(i32, String, bool)>,
    pub reveal_token: i32,
    pub flash: Option<(String, Instant)>,
    /// Album row id -> member message ids (rebuilt with every conversation frame).
    pub albums: HashMap<String, Vec<String>>,
    // overlays
    pub picker_tab: i32,
    pub emoji_tab: i32,
    pub emoji_search: String,
    pub gif: GifState,
    pub open_delete: Option<(String, bool)>,
    pub open_reaction: Option<(String, String)>,
    pub open_album: Option<String>,
    /// One-shot: open the settings panel (demo / shortcuts).
    pub open_settings: bool,
    pub preview: Option<Preview>,
    pub rec: Option<Rec>,
    pub viewer: Option<Viewer>,
}

pub struct Controller {
    pub(crate) api: Api,
    pub rt: tokio::runtime::Handle,
    pub(crate) window: slint::Weak<AppWindow>,
    pub(crate) store: Mutex<Store>,
    pub(crate) settings: Mutex<Settings>,
    pub(crate) view: Mutex<View>,
    pub(crate) images: Mutex<ImageCache>,
    /// Decrypted media already in the cache, by message id.
    pub(crate) media_paths: Mutex<HashMap<String, PathBuf>>,
    pub(crate) media_inflight: Mutex<HashSet<String>>,
    /// Auto-loads that failed this session (not retried until clicked).
    pub(crate) media_failed: Mutex<HashSet<String>>,
    pub(crate) media_permits: Semaphore,
    /// Row the list last asked for (or [`FOCUS_TAIL`]); centres the row window.
    pub(crate) focus: AtomicUsize,
    pub(crate) built_focus: AtomicUsize,
    pub(crate) win_first: AtomicUsize,
    pub(crate) win_len: AtomicUsize,
    pub(crate) win_total: AtomicUsize,
    pub(crate) lid_inflight: Mutex<HashSet<String>>,
    pub(crate) avatars: Mutex<HashMap<String, Pixels>>,
    pub(crate) avatar_fetched: Mutex<HashSet<String>>,
    pub(crate) avatar_permits: Semaphore,
    pub(crate) wallpaper: Mutex<Option<Pixels>>,
    pub(crate) audio: AudioPlayer,
    receipts: Mutex<Vec<(String, Option<String>, String)>>,
    receipts_scheduled: Mutex<bool>,
    acked: Mutex<(HashSet<String>, VecDeque<String>)>,
    dirty: AtomicU8,
    wake: Notify,
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl Controller {
    pub fn new(api: Api, rt: tokio::runtime::Handle, window: slint::Weak<AppWindow>) -> Arc<Self> {
        let settings = Settings::load();
        let emoji_tab = 0;
        let ctrl = Arc::new(Self {
            api,
            rt,
            window,
            store: Mutex::new(Store::new()),
            settings: Mutex::new(settings),
            view: Mutex::new(View {
                loading_label: "Loading your chats…".into(),
                emoji_tab,
                ..Default::default()
            }),
            images: Mutex::new(ImageCache::new(IMAGE_CACHE_BYTES)),
            media_paths: Mutex::new(HashMap::new()),
            media_inflight: Mutex::new(HashSet::new()),
            media_failed: Mutex::new(HashSet::new()),
            media_permits: Semaphore::new(MEDIA_LOADS),
            focus: AtomicUsize::new(FOCUS_TAIL),
            built_focus: AtomicUsize::new(FOCUS_TAIL),
            win_first: AtomicUsize::new(0),
            win_len: AtomicUsize::new(0),
            win_total: AtomicUsize::new(0),
            lid_inflight: Mutex::new(HashSet::new()),
            avatars: Mutex::new(HashMap::new()),
            avatar_fetched: Mutex::new(HashSet::new()),
            avatar_permits: Semaphore::new(4),
            wallpaper: Mutex::new(None),
            audio: AudioPlayer::new(),
            receipts: Mutex::new(Vec::new()),
            receipts_scheduled: Mutex::new(false),
            acked: Mutex::new((HashSet::new(), VecDeque::new())),
            dirty: AtomicU8::new(DIRTY_ALL),
            wake: Notify::new(),
        });
        ctrl.load_wallpaper();
        ctrl
    }

    /// The list instantiated row `i`. Called on the UI thread from the model, so
    /// it only touches atomics; a rebuild is scheduled when the row nears the
    /// window's edge or has moved far enough to change which images are near.
    pub(crate) fn row_requested(&self, i: usize) {
        self.focus.store(i, Ordering::Relaxed);
        let first = self.win_first.load(Ordering::Relaxed);
        let end = first + self.win_len.load(Ordering::Relaxed);
        let total = self.win_total.load(Ordering::Relaxed);
        let low = first > 0 && i < first + WINDOW_MARGIN;
        let high = end < total && i + WINDOW_MARGIN >= end;
        let built = self.built_focus.load(Ordering::Relaxed);
        let moved = built != FOCUS_TAIL && i.abs_diff(built) >= NEAR_STEP;
        let from_tail = built == FOCUS_TAIL && i + NEAR_STEP < total;
        if low || high || moved || from_tail {
            self.mark(DIRTY_CONV);
        }
    }

    #[cfg(test)]
    pub(crate) fn dirty_bits(&self) -> u8 {
        self.dirty.swap(0, Ordering::SeqCst)
    }

    pub(crate) fn mark(&self, bits: u8) {
        self.dirty.fetch_or(bits, Ordering::SeqCst);
        self.wake.notify_one();
    }

    /// Show a toast (5 s, max 3, no duplicates of a visible one).
    pub(crate) fn toast(&self, text: impl Into<String>) {
        let text = text.into();
        let mut v = lock(&self.view);
        if v.toasts.iter().any(|(t, _)| *t == text) {
            return;
        }
        v.toasts.push((text, Instant::now()));
        while v.toasts.len() > TOAST_MAX {
            v.toasts.remove(0);
        }
        drop(v);
        self.mark(DIRTY_SESSION);
    }

    pub fn dismiss_toast(&self, index: usize) {
        let mut v = lock(&self.view);
        if index < v.toasts.len() {
            v.toasts.remove(index);
        }
        drop(v);
        self.mark(DIRTY_SESSION);
    }

    pub(crate) fn active(&self) -> Option<String> {
        lock(&self.store).active.clone()
    }

    pub(crate) fn self_jid(&self) -> Option<String> {
        lock(&self.view).status.jid.clone()
    }

    // ================================================================ loops

    /// Startup: boot the session, reconcile the on-disk cache against the
    /// handshake state, then consume events (port of startEventBridge).
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
            self.hydrate(cached_account).await;
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

    /// Restore the previous run's chats: rows first, then the newest message
    /// of each chat in chunks of 200 (driving the loading screen's progress).
    async fn hydrate(self: &Arc<Self>, cached_account: Option<String>) {
        {
            let mut v = lock(&self.view);
            v.hydrating = true;
            v.loading_label = "Reading saved chats…".into();
        }
        self.mark(DIRTY_SESSION);
        let jid = self.self_jid();
        let different = matches!((&cached_account, &jid), (Some(a), Some(j)) if normalize_jid(a) != normalize_jid(j));
        if different {
            lock(&self.store).clear_all();
        } else {
            let me = self.clone();
            let jids = tokio::task::spawn_blocking(move || {
                let mut s = lock(&me.store);
                s.hydrate_rows()
            })
            .await
            .unwrap_or_default();
            {
                let mut v = lock(&self.view);
                v.loading_label = "Restoring chats…".into();
                v.loading_total = jids.len() as i32;
                v.loading_done = 0;
            }
            self.mark(DIRTY_SESSION);
            for chunk in jids.chunks(200) {
                let me = self.clone();
                let chunk = chunk.to_vec();
                let n = chunk.len() as i32;
                let _ = tokio::task::spawn_blocking(move || lock(&me.store).hydrate_tails(&chunk))
                    .await;
                lock(&self.view).loading_done += n;
                self.mark(DIRTY_SESSION);
                tokio::task::yield_now().await;
            }
            for j in jids.into_iter().filter(|j| is_lid(j)) {
                tokio::spawn(self.clone().unify_lid(j));
            }
        }
        let mut v = lock(&self.view);
        v.hydrating = false;
        v.loading_total = 0;
    }

    /// Demo mode: only the render + housekeeping loops (no session).
    #[cfg(debug_assertions)]
    pub async fn run_render_only(self: Arc<Self>) {
        tokio::spawn(self.clone().housekeeping_loop());
        self.render_loop().await;
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
            if self
                .window
                .upgrade_in_event_loop(move |w| frame.apply(&w))
                .is_err()
            {
                break;
            }
        }
    }

    /// Periodic: persist dirty state, expire toasts/flash/syncing badge, tick
    /// audio progress and the recorder clock.
    async fn housekeeping_loop(self: Arc<Self>) {
        let mut tick = tokio::time::interval(Duration::from_millis(250));
        let mut n: u64 = 0;
        loop {
            tick.tick().await;
            n += 1;
            if n % 4 == 0 {
                let me = self.clone();
                let _ = tokio::task::spawn_blocking(move || lock(&me.store).flush_periodic()).await;
            }
            let mut bits = 0;
            {
                let mut v = lock(&self.view);
                let before = v.toasts.len();
                v.toasts.retain(|(_, t)| t.elapsed() < TOAST_TTL);
                if v.toasts.len() != before {
                    bits |= DIRTY_SESSION;
                }
                let syncing = v
                    .last_history_chunk
                    .is_some_and(|t| t.elapsed() < Duration::from_millis(2500));
                if syncing != v.syncing_shown {
                    v.syncing_shown = syncing;
                    bits |= DIRTY_SESSION | DIRTY_CHATS;
                }
                if v.flash
                    .as_ref()
                    .is_some_and(|(_, t)| t.elapsed() > Duration::from_millis(1500))
                {
                    v.flash = None;
                    bits |= DIRTY_CONV;
                }
                if matches!(v.rec, Some(Rec::Voice(_))) {
                    bits |= DIRTY_OVERLAY;
                }
            }
            if self.audio.snapshot().key.is_some() {
                bits |= DIRTY_CONV | DIRTY_OVERLAY;
            }
            if bits != 0 {
                self.mark(bits);
            }
        }
    }

    /// Final flush on window close.
    pub fn shutdown(&self) {
        self.audio.stop();
        lock(&self.store).flush();
    }

    // ================================================================ session

    pub(crate) async fn refresh_status(&self) {
        match self.api.auth_status().await {
            Ok(s) => {
                lock(&self.view).status = s;
                self.learn_self_lid();
            }
            Err(e) => log::error!("auth_status failed: {e}"),
        }
        self.mark(DIRTY_SESSION | DIRTY_CHATS);
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
    /// QR can be generated (logout / "Stuck? Log out & reset").
    pub(crate) async fn reset_all(&self) {
        {
            let mut v = lock(&self.view);
            let booted = v.booted;
            *v = View {
                booted,
                resetting: true,
                loading_label: "Loading your chats…".into(),
                ..Default::default()
            };
        }
        self.audio.stop();
        lock(&self.store).clear_all();
        lock(&self.media_paths).clear();
        lock(&self.avatars).clear();
        lock(&self.avatar_fetched).clear();
        self.mark(DIRTY_ALL);
        if let Err(e) = self.api.reset_session().await {
            log::error!("reset session failed: {e}");
        }
        lock(&self.view).resetting = false;
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
                            let who = if cm.dto.sender_jid.is_empty() {
                                cm.dto.chat_jid.clone()
                            } else {
                                cm.dto.sender_jid.clone()
                            };
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
                    // Mapping not learned yet: resolve first, then apply under
                    // whatever key wins.
                    self.clone().unify_lid(receipt.chat_jid.clone()).await;
                    jid = lock(&self.store).canonical(&receipt.chat_jid);
                }
                lock(&self.store).apply_receipt(
                    &jid,
                    &receipt.message_ids,
                    MessageStatus::parse(&receipt.status),
                );
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
                    (
                        s.canonical(&update.chat_jid),
                        s.canonical(update.sender_jid.as_deref().unwrap_or("")),
                    )
                };
                {
                    let mut s = lock(&self.store);
                    match update.kind.as_str() {
                        "revoke" => s.apply_revoke_event(
                            &jid,
                            &update.target_id,
                            &sender,
                            update.from_me,
                            update.timestamp,
                        ),
                        "edit" => s.apply_edit(
                            &jid,
                            &update.target_id,
                            update.text.clone(),
                            update.timestamp,
                        ),
                        "reaction" => {
                            let reactor = if update.from_me {
                                self.self_jid()
                                    .map(|j| normalize_jid(&j))
                                    .unwrap_or_else(|| normalize_jid(&sender))
                            } else {
                                normalize_jid(&sender)
                            };
                            s.apply_reaction(
                                &jid,
                                &update.target_id,
                                &reactor,
                                update.text.as_deref().unwrap_or(""),
                            );
                        }
                        _ => {}
                    }
                }
                tokio::spawn(self.clone().unify_lid(update.chat_jid.clone()));
                if let Some(sj) = update.sender_jid.filter(|s| *s != update.chat_jid) {
                    tokio::spawn(self.clone().unify_lid(sj));
                }
                self.mark_jid(&jid);
            }
            UiEvent::DeleteForMe {
                chat_jid,
                message_id,
                ..
            } => {
                let jid = lock(&self.store).canonical(&chat_jid);
                lock(&self.store).remove_message(&jid, &message_id);
                tokio::spawn(self.clone().unify_lid(chat_jid));
                self.mark_jid(&jid);
            }
            UiEvent::StreamError { message, .. } => {
                log::warn!("stream error: {message}");
            }
        }
    }

    pub(crate) fn mark_jid(&self, jid: &str) {
        let active = self.active().as_deref() == Some(jid);
        self.mark(DIRTY_CHATS | if active { DIRTY_CONV } else { 0 });
    }

    pub(crate) fn on_message(self: &Arc<Self>, m: MessageDto) {
        let raw_chat = m.chat_jid.clone();
        let raw_sender = m.sender_jid.clone();
        let jid = {
            let mut s = lock(&self.store);
            let cm = s.canonical_message(m);
            let jid = cm.dto.chat_jid.clone();
            if let (Some(n), false) = (cm.dto.push_name.clone(), cm.dto.from_me) {
                // A DM's name fills in from the sender's pushName (never overwrites).
                if !is_group(&jid)
                    && let Some(c) = s.chats.get_mut(&jid)
                    && c.name.is_none()
                {
                    c.name = Some(n.clone());
                }
                let who = if cm.dto.sender_jid.is_empty() {
                    jid.clone()
                } else {
                    cm.dto.sender_jid.clone()
                };
                s.learn_name(&who, &n);
            }
            s.add_message(cm);
            jid
        };
        tokio::spawn(self.clone().unify_lid(raw_chat.clone()));
        if raw_sender != raw_chat {
            tokio::spawn(self.clone().unify_lid(raw_sender));
        }
        self.mark_jid(&jid);
    }

    /// Ensure a LID jid is unified with its PN form (port of lid.ts unifyLid).
    pub(crate) async fn unify_lid(self: Arc<Self>, jid: String) {
        if !is_lid(&jid) {
            return;
        }
        let jid = normalize_jid(&jid);
        if lock(&self.store).knows_lid(&jid) {
            let pn = lock(&self.store).canonical(&jid);
            lock(&self.store).learn_lid(&jid, &pn);
            return;
        }
        if !lock(&self.lid_inflight).insert(jid.clone()) {
            return;
        }
        match self.api.resolve_jid(jid.clone()).await {
            Ok(r) => {
                // A miss is not cached: a later event retries.
                if let Some(pn) = r.pn.filter(|p| *p != jid) {
                    lock(&self.store).learn_lid(&jid, &pn);
                    self.mark(DIRTY_CHATS | DIRTY_CONV);
                }
            }
            Err(e) => log::debug!("lid resolve failed for {jid}: {e}"),
        }
        lock(&self.lid_inflight).remove(&jid);
    }

    // ================================================================ read receipts

    /// A message (or album) scrolled into view: clear its unread flag, count
    /// the chat down (marking the whole chat read at zero) and, when enabled,
    /// queue a read receipt (batched, deduplicated).
    pub fn seen(self: &Arc<Self>, id: String) {
        let Some(jid) = self.active() else { return };
        let ids = lock(&self.view)
            .albums
            .get(&id)
            .cloned()
            .unwrap_or_else(|| vec![id]);
        let send = lock(&self.settings).send_read_receipts;
        let mut chat_read = false;
        let mut any = false;
        for id in ids {
            let (result, wire) = {
                let mut s = lock(&self.store);
                let wire = s.find_message(&jid, &id).map(|m| {
                    (
                        m.proto_chat_jid().to_string(),
                        is_group(&jid).then(|| m.proto_sender_jid().to_string()),
                    )
                });
                (s.mark_message_read(&jid, &id), wire)
            };
            let Some(zero) = result else { continue };
            any = true;
            chat_read |= zero;
            if send && let Some((wire_chat, sender)) = wire {
                let key = format!("{jid}|{id}");
                let mut acked = lock(&self.acked);
                if acked.0.insert(key.clone()) {
                    acked.1.push_back(key);
                    if acked.1.len() > ACKED_MAX
                        && let Some(old) = acked.1.pop_front()
                    {
                        acked.0.remove(&old);
                    }
                    lock(&self.receipts).push((wire_chat, sender, id));
                }
            }
        }
        if !any {
            return;
        }
        self.mark(DIRTY_CHATS | DIRTY_CONV);
        if chat_read {
            let me = self.clone();
            let chat = jid.clone();
            tokio::spawn(async move {
                if let Err(e) = me.api.mark_read(chat).await {
                    log::warn!("mark read failed: {e}");
                }
            });
        }
        let mut scheduled = lock(&self.receipts_scheduled);
        if !*scheduled && !lock(&self.receipts).is_empty() {
            *scheduled = true;
            let me = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(RECEIPT_DEBOUNCE).await;
                *lock(&me.receipts_scheduled) = false;
                me.flush_receipts().await;
            });
        }
    }

    async fn flush_receipts(&self) {
        let queue = std::mem::take(&mut *lock(&self.receipts));
        let mut groups: HashMap<(String, Option<String>), Vec<String>> = HashMap::new();
        for (chat, sender, id) in queue {
            groups.entry((chat, sender)).or_default().push(id);
        }
        for ((chat, sender), ids) in groups {
            if let Err(e) = self.api.mark_read_messages(chat, sender, ids).await {
                log::warn!("read receipts failed: {e}");
            }
        }
    }
}

/// The controller, reachable from detached tasks spawned by `&self` methods.
pub static CTRL: std::sync::OnceLock<Arc<Controller>> = std::sync::OnceLock::new();
