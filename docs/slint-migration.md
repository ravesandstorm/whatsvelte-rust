# Tauri + Svelte → Slint migration plan

Goal: replace the WebView frontend (Tauri shell + Svelte SPA) with a native
[Slint](https://slint.dev) UI, so the app ships as one Rust binary with no
WebView, no JS heap and no IPC serialization. `rust-backend/` (the
`whatsapp-rust` library) is **not touched** — it stays a path dependency.

## Target layout

```
slint-client/                 new crate, replaces src-tauri/ + svelte-frontend/
  Cargo.toml / build.rs       slint-build compiles ui/app.slint
  ui/                         .slint markup (one file per screen/component)
  src/
    main.rs                   tokio runtime + Slint event loop
    core/                     Tauri-free port of src-tauri/src
      session.rs              SessionManager (unchanged logic, no AppHandle)
      bridge.rs               Event -> typed UiEvent (no JSON envelopes)
      dto.rs, error.rs        same shapes, serde kept for on-disk cache
      api.rs                  former #[tauri::command]s as plain async fns
      media.rs                download/cache/save/send media
      paths.rs                data/cache/downloads dirs (replaces app.path())
    store/                    port of svelte-frontend/src/lib/stores
      chats.rs, messages.rs   chat rows + per-chat message windows
      lid.rs, names.rs        LID<->PN unification, learned push names
      persist.rs              JSON files on disk (replaces IndexedDB/localStorage)
      settings.rs             theme, zoom, read receipts
    ui/                       glue between store and Slint models
      controller.rs           event pump, debounced model refresh, callbacks
      images.rs               base64/JPEG/WebP -> SharedPixelBuffer (off UI thread)
      qr.rs                   QR code -> pixel buffer
    util/jid.rs               port of lib/util/jid.ts
```

## Mapping

| Tauri/Svelte                         | Slint client                                        |
|--------------------------------------|-----------------------------------------------------|
| `#[tauri::command]` + `invoke()`     | `core::api` async fn, called from a Slint callback  |
| `app.emit("wa://…")` + `listen()`    | `UiEvent` over an `async_channel`, applied to store  |
| Svelte runes / `SvelteMap`           | `store::*` structs + `VecModel` pushed to `AppState` |
| IndexedDB / localStorage             | JSON files under `<data_dir>/ui-cache/<account>/`    |
| asset protocol (`convertFileSrc`)    | decode cached file -> `slint::Image`                 |
| `<video>` / `<audio>` / docs         | open in the OS default app (`open` crate)            |
| `tauri-plugin-dialog`                | `rfd` native file dialog                             |
| `tauri-plugin-opener`                | `open` crate                                         |
| QR `<canvas>`                        | `qrcode` crate -> pixel buffer                       |

## Phases (each lands as its own commit)

1. **Plan** — this document.
2. **Scaffold** — `slint-client` crate, build.rs, empty window, tokio runtime.
3. **Core port** — session/bridge/dto/error/api/media without Tauri types.
4. **Store** — chats, messages, LID map, names, settings, JSON persistence.
5. **UI: pairing + loading** — QR, pair code, connection/hydration states.
6. **UI: main layout** — chat list (sections: chats/archived/channels/status),
   conversation view, message bubbles (text, quotes, media thumbs, ticks,
   reactions, edited/deleted), composer (text, reply, edit).
7. **UI: actions** — context menu (reply/react/edit/delete), chat flags
   (mute/pin/archive), media open/save, attach + send media, settings panel.
8. **Tooling** — Makefile, release scripts, CI workflow switched to the Slint
   build; README/ARCHITECTURE updated.
9. **Retire** — `src-tauri/`, `svelte-frontend/` and the root npm files were
   removed once the parity checklist below was verified (done).

## Feature parity checklist

Every user-visible feature of the Tauri + Svelte frontend, and where it lives
now. ✅ = ported with the same behaviour, ➕ = ported and extended,
↪ = deliberately different (agreed trade-off).

| Area | Svelte feature | Slint client |
|------|----------------|--------------|
| Pairing | QR (re-rendered per `auth/qr`), "Waiting for QR…", expired-QR state, connection pill | ✅ `ui/pairing.slint`, `ui/qr.rs` ➕ phone-number pair code |
| Loading | hydrate label, "N of M chats · %" progress (200-chat chunks), "Stuck? Log out & reset" | ✅ `ui/loading.slint`, `Controller::hydrate` |
| Boot | registered (IK/XX) is login truth; wipe stale cache when logged out; account switch wipes; LID re-key on hydrate | ✅ `Controller::run`, `Store::hydrate_rows` |
| Chat list | search (title / number / last message), pills All · Channels · Status · Archived (counts, archive bounce), pinned first, drafts in red, group sender prefix + ticks, unread badge, "(You)" self chat, empty states | ✅ `ui/chat_list.slint`, `views.rs::chats_frame` |
| Chat list | right-click Pin / Mute / Archive (optimistic, revert on error) | ✅ `Controller::chat_action` |
| Avatars | lazy `get_contact` picture per visible row, initials fallback, "#" for groups | ✅ `Controller::need_avatar` |
| Messages | 40-row window, scroll-up paging with position restore, release pages at bottom, "Showing the latest messages" hint | ✅ `ui/conversation.slint` |
| Messages | unread flags, "Unread" divider, open at first unread, jump-to-bottom with badge, mark read when seen | ✅ `Store::mark_message_read`, `Controller::seen` |
| Receipts | batched (600 ms) `mark_read_messages` grouped by chat/sender, 5000-entry dedupe, whole-chat `mark_read` at zero, setting to disable | ✅ `Controller::seen` / `flush_receipts` |
| Bubbles | sender grouping, group sender names, quotes (with thumb, click → jump + 1.5 s flash), reactions chips, edited, ticks (sending / failed / sent / delivered / read), tombstones, structured labels, stickers without chrome | ✅ `ui/conversation.slint::Bubble`, `views.rs::conv_frame` |
| Links | `https?://…` / `www.…` with trailing punctuation stripped, open in OS browser | ✅ `StyledText` + `views.rs::linkify_markdown` |
| Albums | explicit (`albumParentId`, ≥2) and inferred (≥4, ≤60 s apart) grouping, 2×2 grid with "+N", album sheet | ✅ `views.rs::album_len`, `AlbumGrid`, `AlbumSheet` |
| Lightbox | thumbnail then full image, wheel zoom 1–8× + drag pan, gallery ‹ › / arrow keys, "i / N", Download → "✓ Saved", Esc | ✅ `Lightbox` |
| Video | poster + ▶ | ↪ plays in the OS default player (no in-app decoder) |
| Audio | download on click, play / pause, one at a time | ✅ `media/audio.rs` (rodio + libopus) |
| Documents | open in OS app, ⬇ save to Downloads | ✅ |
| Context menu | Reply / Edit / Delete (+ React) | ➕ also Copy text, Open, Save |
| Delete | modal: for everyone (own) / for me / cancel, optimistic with rollback toasts | ✅ `DeleteModal` |
| Reactions | quick row 👍❤️😂😮😢🙏 + full picker, toggle-off, optimistic + rollback | ✅ `ReactionPicker` |
| Composer | multiline, Enter / Shift+Enter / Ctrl·Cmd+Enter, Esc cancels edit then reply, drafts, edit stash restore, reply banner with thumb | ✅ |
| Shortcodes | `:token` popover (8 results, prefix first), ↑ ↓ Enter Tab Esc | ✅ `util/emoji.rs` |
| Emoji | 5 categories (1088 emoji), 605 shortcodes, search (64), recents (32) | ✅ |
| GIFs | Klipy trending / search, 350 ms debounce, infinite scroll, send MP4 with gif-playback (else GIF) | ✅ `media/klipy.rs` ↪ previews are static first frames |
| Stickers tab | "Stickers coming soon" | ✅ (same placeholder) |
| Attach | Photos (filtered), Document, Camera, Voice | ✅ ↪ "Record video" dropped (agreed) |
| Preview | caption, HD toggle (not GIFs), Enter sends, Esc cancels | ✅ `MediaPreview` |
| Sending media | JPEG 1600 px q75 / HD 3000 px q92, GIF passthrough, 200 px q60 thumbs, temp staging, labels | ✅ `media/prep.rs` |
| Voice notes | MediaRecorder → raw webm | ➕ Ogg/Opus 48 kHz mono, sent as PTT |
| Paste | image from clipboard → preview | ✅ (arboard) ➕ drag-and-drop files |
| Settings | Light / Dark (default dark), zoom 0.5–2.0 step 0.1 (buttons + Ctrl/Cmd +/−/0), wallpapers Default / Teal / Slate / Dusk / custom image, Enter sends, read receipts, log out | ✅ ➕ System theme, image auto-load |
| Toasts | 5 s, max 3, dedupe, click to dismiss | ✅ |
| LID ↔ PN | canonical keys, wire jids for protocol calls, on-demand resolve + merge, receipts wait for resolve | ✅ `Store::learn_lid`, `Controller::unify_lid` |
| Persistence | IndexedDB / localStorage | ✅ JSON files under app data (`store/persist.rs`) |
| Newsletters | fetched once per connected session, retried on failure | ✅ |

### Verification notes

Unit tests cover the store (dedupe, receipts, LID merge, unread flags, revoke
tombstones), album grouping, link detection, Opus round-trip, image
compression, Klipy parsing and settings. `WA_DEMO=1` (debug builds) renders
sample chats offline. `WA_DEMO_OPEN=reactions|delete|album|viewer|reply|suggest`
opens one overlay, and `WA_SNAPSHOT=out.png` saves a window render.
Scroll-position behaviours (open at the unread divider, quote jump,
jump-to-bottom, paging restore) and device capture (camera / microphone
permissions) need a manual pass on a visible window.
