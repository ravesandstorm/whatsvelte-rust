//! Mock backend (debug builds only, enabled with `WA_MOCK=1`; see `make mock`).
//!
//! Same command names and `wa://` events as the real backend, so the frontend
//! runs unmodified, in the real webview, over real IPC. No session or SQLite is
//! created and nothing touches the network.
//!
//! `make mock` serves the frontend from its own port, which gives it its own
//! IndexedDB origin: the real app's cached chats are never read or wiped.
//!
//! Tunables (env): `WA_MOCK_CHATS` (default 80), `WA_MOCK_MSGS` (default 300).
//!
//! Control channel, so scripts can drive the real webview: connect to
//! 127.0.0.1:5177, send one line of JS, and read back one line of JSON. The JS
//! is evaluated in the page, and a returned promise is awaited:
//!   printf 'document.title\n' | nc 127.0.0.1 5177
//! The `mock_incoming` and `mock_history` commands simulate live traffic, and
//! `mock_wheel` (macOS) sends the webview real scroll events.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::dto::{
    ChatDto, ContactDto, HistoryDto, MessageDto, PairCodeDto, QuotedDto, ReceiptDto, ResolveJidDto,
    SendResultDto, StatusDto,
};
use crate::error::{ApiError, ApiResult};

const SESSION: &str = "default";
const ME: &str = "919999900000@s.whatsapp.net";
const CONTROL_ADDR: &str = "127.0.0.1:5177";

const NAMES: &[&str] = &[
    "Aarav", "Diya", "Kabir", "Meera", "Rohan", "Isha", "Vihaan", "Anaya", "Arjun", "Sara", "Neil",
    "Tara", "Dev", "Zoya", "Kian", "Riya", "Om", "Nisha", "Yash", "Pooja",
];
const LINES: &[&str] = &[
    "Hey, are you free later?",
    "Sent you the doc, take a look",
    "lol that's hilarious",
    "Can we move the call to 4?",
    "Thanks! 🙏",
    "Running 10 mins late",
    "Did you see the news today? It's honestly wild how fast things are changing, I can't keep up with any of it anymore.",
    "ok",
    "Let me check and get back to you",
    "Sounds good to me 👍",
    "Which one did you pick in the end? I couldn't decide between the two.",
    "Happy birthday!! 🎉🎂",
    "Long one, sorry.\nFirst, the venue moved to the other building.\nSecond, bring your ID this time.\nThird, we start at 9 sharp, so please don't be late.",
];

static IDS: AtomicU64 = AtomicU64::new(0);
static HISTORY_SENT: AtomicBool = AtomicBool::new(false);

fn pending_evals() -> &'static Mutex<HashMap<u64, Sender<String>>> {
    static PENDING: OnceLock<Mutex<HashMap<u64, Sender<String>>>> = OnceLock::new();
    PENDING.get_or_init(Default::default)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Deterministic LCG: the same ids and content on every launch, so a relaunch
/// merges with what the frontend already cached instead of duplicating it.
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(self.0) / 4_294_967_296.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() * n as f64) as usize
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

fn next_id(prefix: &str) -> String {
    format!("{prefix}{:010X}", IDS.fetch_add(1, Ordering::Relaxed))
}

fn chat_jid(i: usize) -> String {
    if i % 9 == 8 {
        format!("1203630{i}@g.us")
    } else {
        format!("9198000{i:05}@s.whatsapp.net")
    }
}

fn make_message(rng: &mut Rng, jid: &str, timestamp: i64, from_me: bool, text: Option<String>) -> MessageDto {
    let sender = if from_me {
        ME.to_string()
    } else if jid.ends_with("@g.us") {
        format!("9198111{:05}@s.whatsapp.net", rng.below(5))
    } else {
        jid.to_string()
    };
    MessageDto {
        id: next_id("MOCK"),
        chat_jid: jid.to_string(),
        sender_jid: sender,
        from_me,
        timestamp,
        push_name: if from_me { None } else { Some(rng.pick(NAMES).to_string()) },
        text: Some(text.unwrap_or_else(|| rng.pick(LINES).to_string())),
        kind: "text".into(),
        thumbnail: None,
        media: None,
        quoted: None,
        album_parent_id: None,
    }
}

/// One conversation, the same granularity the real history sync emits.
fn build_conversation(rng: &mut Rng, i: usize, per_chat: usize, base: i64) -> HistoryDto {
    let jid = chat_jid(i);
    // The first chat is always long enough to exercise paging from IndexedDB.
    let count = if i == 0 { per_chat.max(600) } else { (rng.below(per_chat.max(1))).max(5) };
    let start = base - (i as i64) * 3600 - (count as i64) * 90;
    let mut messages: Vec<MessageDto> = Vec::with_capacity(count);
    for k in 0..count {
        let from_me = rng.next() < 0.4;
        let mut m = make_message(rng, &jid, start + (k as i64) * 90, from_me, None);
        // Occasional replies, for bubbles of a different height.
        if k > 0 && k % 17 == 0 {
            let q = &messages[k - 1];
            m.quoted = Some(QuotedDto {
                id: q.id.clone(),
                sender_jid: Some(q.sender_jid.clone()),
                text: q.text.clone(),
                kind: "text".into(),
                thumbnail: None,
            });
        }
        messages.push(m);
    }
    let last = &messages[messages.len() - 1];
    let chat = ChatDto {
        jid: jid.clone(),
        name: Some(if jid.ends_with("@g.us") {
            format!("Group {i}")
        } else {
            format!("{} {i}", NAMES[i % NAMES.len()])
        }),
        last_message: last.text.clone(),
        timestamp: last.timestamp,
        unread: if i % 5 == 1 { 3 } else { 0 },
    };
    HistoryDto { chats: vec![chat], messages }
}

fn emit(app: &AppHandle, topic: &str, kind: &str, payload: impl serde::Serialize) {
    let envelope = json!({ "sessionId": SESSION, "kind": kind, "payload": payload });
    if let Err(e) = app.emit(topic, envelope) {
        log::error!("mock emit {topic} failed: {e}");
    }
}

/// Stream the history, one conversation per chunk with a small gap, as the real
/// decoder does.
fn stream_history(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let chats = env_usize("WA_MOCK_CHATS", 80);
        let per_chat = env_usize("WA_MOCK_MSGS", 300);
        let base = now();
        let mut rng = Rng(42);
        for i in 0..chats {
            let chunk = build_conversation(&mut rng, i, per_chat, base);
            emit(&app, "wa://history", "HistorySync", chunk);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        log::info!("[mock] history sent: {chats} conversation(s)");
    });
}

/// Start the control channel. Call once, from `setup`.
pub fn start_control_channel(app: AppHandle) {
    let listener = match TcpListener::bind(CONTROL_ADDR) {
        Ok(l) => l,
        Err(e) => {
            log::error!("[mock] control channel unavailable on {CONTROL_ADDR}: {e}");
            return;
        }
    };
    log::info!("[mock] control channel on {CONTROL_ADDR}");
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let app = app.clone();
            std::thread::spawn(move || {
                if let Err(e) = serve_control(app, stream) {
                    log::warn!("[mock] control connection: {e}");
                }
            });
        }
    });
}

fn serve_control(app: AppHandle, stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        let reply = eval_in_page(&app, line.trim());
        writeln!(writer, "{reply}")?;
        line.clear();
    }
    Ok(())
}

/// Evaluate `code` in the page and wait for `mock_eval_result` to bring back its
/// value. `__TAURI_INTERNALS__` is always injected, unlike `window.__TAURI__`.
fn eval_in_page(app: &AppHandle, code: &str) -> String {
    let Some(window) = app.get_webview_window("main") else {
        return json!({ "error": "no main window" }).to_string();
    };
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = channel();
    pending_evals().lock().unwrap().insert(id, tx);
    let script = format!(
        "(async () => {{ let out; \
           try {{ out = {{ value: await (0, eval)({code}) }}; }} \
           catch (e) {{ out = {{ error: String(e) }}; }} \
           await window.__TAURI_INTERNALS__.invoke('mock_eval_result', {{ id: {id}, result: JSON.stringify(out) ?? '{{}}' }}); \
         }})()",
        code = Value::String(code.to_string()),
    );
    if let Err(e) = window.eval(script) {
        pending_evals().lock().unwrap().remove(&id);
        return json!({ "error": e.to_string() }).to_string();
    }
    let reply = rx.recv_timeout(Duration::from_secs(60));
    pending_evals().lock().unwrap().remove(&id);
    reply.unwrap_or_else(|_| json!({ "error": "timed out waiting for the page" }).to_string())
}

/// Commands here stay private and are registered from this module: a `pub` one
/// exports its generated macro at the crate root, where it would collide with
/// the real command of the same name.
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        auth_status,
        auth_start_qr,
        auth_start_pair_code,
        connect,
        disconnect,
        auth_logout,
        reset_session,
        send_text,
        send_reply,
        edit_message,
        revoke_message,
        delete_for_me,
        send_reaction,
        mark_read,
        mark_read_messages,
        get_contact,
        get_profile_picture_url,
        resolve_jid,
        download_media,
        save_media_to_downloads,
        send_media,
        set_chat_muted,
        set_chat_pinned,
        set_chat_archived,
        list_newsletters,
        mock_eval_result,
        mock_incoming,
        mock_history,
        mock_wheel,
        mock_raise,
    ]
}

// ---- Mock-only commands ----------------------------------------------------

#[tauri::command]
fn mock_eval_result(id: u64, result: String) {
    if let Some(tx) = pending_evals().lock().unwrap().remove(&id) {
        let _ = tx.send(result);
    }
}

/// Simulate a live incoming message (an update at the bottom of the list).
#[tauri::command]
fn mock_incoming(jid: Option<String>, text: Option<String>, app: AppHandle) {
    let jid = jid.unwrap_or_else(|| chat_jid(0));
    let mut rng = Rng(IDS.load(Ordering::Relaxed) as u32);
    let msg = make_message(&mut rng, &jid, now(), false, text);
    emit(&app, "wa://message", "Message", msg);
}

/// Simulate a history chunk of messages older than `before` (an update at the top).
#[tauri::command]
fn mock_history(jid: Option<String>, count: Option<usize>, before: i64, app: AppHandle) {
    let jid = jid.unwrap_or_else(|| chat_jid(0));
    let count = count.unwrap_or(40);
    let mut rng = Rng(IDS.load(Ordering::Relaxed) as u32);
    let messages = (0..count)
        .map(|k| make_message(&mut rng, &jid, before - ((count - k) as i64) * 90, false, None))
        .collect();
    emit(&app, "wa://history", "HistorySync", HistoryDto { chats: vec![], messages });
}

/// Keep the window on top (or stop). WebKit stops animation frames for a covered
/// window, so a test that needs them has to raise it first.
#[tauri::command]
fn mock_raise(on: bool, app: AppHandle) -> ApiResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| ApiError::Library("no main window".into()))?;
    window.set_always_on_top(on).map_err(ApiError::library)?;
    if on {
        window.show().map_err(ApiError::library)?;
        window.set_focus().map_err(ApiError::library)?;
    }
    Ok(())
}

/// Send `count` native scroll events to the webview at a CSS point, `gap_ms`
/// apart. `continuous` false is a mouse wheel, where `delta` is lines and may be
/// fractional: macOS mouse acceleration reports about 0.1 for a slow notch.
/// `continuous` true is trackpad-like, with `delta` in whole pixels. Negative
/// `delta` scrolls down.
/// `view_height` is `innerHeight`, for flipping `y` to AppKit's origin.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn mock_wheel(
    x: f64,
    y: f64,
    view_height: f64,
    delta: f64,
    count: u32,
    gap_ms: u64,
    continuous: bool,
    app: AppHandle,
) -> ApiResult<()> {
    #[cfg(target_os = "macos")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| ApiError::Library("no main window".into()))?;
        std::thread::spawn(move || {
            for _ in 0..count {
                let posted = window.with_webview(move |webview| {
                    native_wheel::send(webview.inner().cast(), x, view_height - y, delta, continuous);
                });
                if let Err(e) = posted {
                    log::error!("[mock] wheel failed: {e}");
                    return;
                }
                std::thread::sleep(Duration::from_millis(gap_ms));
            }
        });
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (x, y, view_height, delta, count, gap_ms, continuous, app);
        Err(ApiError::Library("mock_wheel is macOS-only".into()))
    }
}

#[cfg(target_os = "macos")]
mod native_wheel {
    use std::ffi::c_void;

    use objc2::encode::{Encoding, RefEncode};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    #[repr(C)]
    struct CgEvent {
        _private: [u8; 0],
    }
    // SAFETY: CGEventRef is `struct __CGEvent *`, only ever used behind a pointer.
    unsafe impl RefEncode for CgEvent {
        const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("__CGEvent", &[]));
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CgPoint {
        x: f64,
        y: f64,
    }

    const UNIT_PIXEL: u32 = 0;
    const UNIT_LINE: u32 = 1;
    /// kCGScrollWheelEventFixedPtDeltaAxis1: the value behind `NSEvent.deltaY`.
    const FIELD_FIXED_PT_DELTA_AXIS_1: u32 = 93;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventCreateScrollWheelEvent2(
            source: *const c_void,
            units: u32,
            wheel_count: u32,
            wheel1: i32,
            wheel2: i32,
            wheel3: i32,
        ) -> *mut CgEvent;
        fn CGEventSetLocation(event: *mut CgEvent, location: CgPoint);
        fn CGEventSetDoubleValueField(event: *mut CgEvent, field: u32, value: f64);
        fn CFRelease(object: *const c_void);
    }

    /// Runs on the main thread. A window-less NSEvent's `locationInWindow` is
    /// the CGEvent location with y flipped, and WebKit only reads it as a window
    /// point, so aim the CGEvent at the flip of the point we want. `event` is
    /// autoreleased.
    pub fn send(webview: *mut AnyObject, x: f64, y_from_bottom: f64, delta: f64, continuous: bool) {
        let units = if continuous { UNIT_PIXEL } else { UNIT_LINE };
        // SAFETY: `webview` is the live WKWebView handed over by `with_webview`,
        // and the CGEvent lives until the CFRelease below.
        unsafe {
            let whole = delta.round() as i32;
            let cg = CGEventCreateScrollWheelEvent2(std::ptr::null(), units, 1, whole, 0, 0);
            if cg.is_null() {
                return;
            }
            if !continuous {
                CGEventSetDoubleValueField(cg, FIELD_FIXED_PT_DELTA_AXIS_1, delta);
            }
            let screen: *mut AnyObject = msg_send![class!(NSScreen), mainScreen];
            let height: f64 = {
                #[repr(C)]
                #[derive(Clone, Copy)]
                struct Rect(f64, f64, f64, f64);
                unsafe impl objc2::encode::Encode for Rect {
                    const ENCODING: Encoding = Encoding::Struct(
                        "CGRect",
                        &[
                            Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]),
                            Encoding::Struct("CGSize", &[Encoding::Double, Encoding::Double]),
                        ],
                    );
                }
                let frame: Rect = msg_send![screen, frame];
                frame.3
            };
            CGEventSetLocation(cg, CgPoint { x, y: height - y_from_bottom });
            let event: *mut AnyObject = msg_send![class!(NSEvent), eventWithCGEvent: cg];
            if !event.is_null() {
                let _: () = msg_send![webview, scrollWheel: event];
            }
            CFRelease(cg.cast());
        }
    }
}

// ---- Same surface as the real commands -------------------------------------

/// The event bridge calls this only after registering its listeners, so the
/// first call is the point where the history can safely start.
#[tauri::command]
fn auth_status(app: AppHandle) -> ApiResult<StatusDto> {
    if !HISTORY_SENT.swap(true, Ordering::SeqCst) {
        stream_history(app);
    }
    Ok(StatusDto {
        session_id: SESSION.into(),
        logged_in: true,
        connected: true,
        jid: Some(ME.into()),
        lid: None,
        registered: true,
        push_name: Some("Me (mock)".into()),
    })
}

/// Sends succeed, then the receipts arrive as they would from the server.
fn accept_send(jid: String, app: AppHandle) -> ApiResult<SendResultDto> {
    let message_id = next_id("MOCKSENT");
    let (chat, id) = (jid.clone(), message_id.clone());
    tauri::async_runtime::spawn(async move {
        for (delay_ms, status) in [(300, "delivered"), (1200, "read")] {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            let receipt = ReceiptDto {
                chat_jid: chat.clone(),
                sender_jid: chat.clone(),
                message_ids: vec![id.clone()],
                status: status.into(),
                timestamp: now(),
            };
            emit(&app, "wa://receipt", "Receipt", receipt);
        }
    });
    Ok(SendResultDto { session_id: SESSION.into(), message_id, to: jid })
}

#[tauri::command]
fn send_text(jid: String, app: AppHandle) -> ApiResult<SendResultDto> {
    accept_send(jid, app)
}

#[tauri::command]
fn send_reply(jid: String, app: AppHandle) -> ApiResult<SendResultDto> {
    accept_send(jid, app)
}

#[tauri::command]
fn edit_message(jid: String, app: AppHandle) -> ApiResult<SendResultDto> {
    accept_send(jid, app)
}

#[tauri::command]
fn send_media(jid: String, app: AppHandle) -> ApiResult<SendResultDto> {
    accept_send(jid, app)
}

#[tauri::command]
fn get_contact(jid: String) -> ApiResult<ContactDto> {
    Ok(ContactDto { jid, name: None, verified_name: None, lid: None, picture_url: None })
}

#[tauri::command]
fn resolve_jid(jid: String) -> ApiResult<ResolveJidDto> {
    Ok(ResolveJidDto { pn: Some(jid), lid: None })
}

#[tauri::command]
fn get_profile_picture_url() -> ApiResult<Option<String>> {
    Ok(None)
}

#[tauri::command]
fn list_newsletters() -> ApiResult<Vec<ChatDto>> {
    Ok(vec![])
}

#[tauri::command]
fn auth_start_pair_code() -> ApiResult<PairCodeDto> {
    Ok(PairCodeDto { session_id: SESSION.into(), code: "MOCK1234".into() })
}

#[tauri::command]
fn download_media() -> ApiResult<String> {
    Err(ApiError::Library("media is not available in mock mode".into()))
}

#[tauri::command]
fn save_media_to_downloads() -> ApiResult<String> {
    Err(ApiError::Library("media is not available in mock mode".into()))
}

macro_rules! no_ops {
    ($($name:ident),* $(,)?) => {
        $(#[tauri::command] fn $name() -> ApiResult<()> { Ok(()) })*
    };
}

no_ops!(
    auth_start_qr,
    connect,
    disconnect,
    auth_logout,
    reset_session,
    revoke_message,
    delete_for_me,
    send_reaction,
    mark_read,
    mark_read_messages,
    set_chat_muted,
    set_chat_pinned,
    set_chat_archived,
);
