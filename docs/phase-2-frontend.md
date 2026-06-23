# Phase 2 — Frontend (Svelte 5 SPA)

> Companion to [`ARCHITECTURE.md`](./ARCHITECTURE.md). This document records what
> the Phase 2 frontend *is*, how its data flows, and what remains. Phase 2 is
> split into **Part A** (the MVP — shipped) and **Part B** (the rich WhatsApp
> object model — planned).

---

## 0. The constraint that shapes everything

The vendored `whatsapp-rust` library is **event-driven and exposes no query API**
for past chats/messages. History arrives exactly once — a `HistorySync` burst at
pairing time — and after that only live `Message`/update events flow. The backend
keeps the *session* in SQLite but never re-serves old conversations.

Consequence: the chat list and message history are a **frontend-owned
projection** of the event stream. Phase 2A's job is to (1) normalize those events
into clean DTOs, (2) reconstruct chats/messages in reactive Svelte stores, and
(3) persist that projection in the webview so a plain restart isn't empty — all
**without modifying the backend SQLite framework**.

---

## Part A — MVP / basic texting  ✅ done

### A.1 Stack & wiring
- **Svelte 5 + Vite + TypeScript** SPA (plain Svelte, not SvelteKit) in
  `svelte-frontend/`, mounted in `main.ts` via `mount(App, …)`.
- Talks to the Phase 1 backend through `@tauri-apps/api`: `invoke()` for
  commands, `listen()` for events. `tauri dev` drives Vite via
  `beforeDevCommand` on `devUrl :5173`.
- `App.svelte` is the top-level switch: `PairingScreen` when `!session.loggedIn`,
  else `MainLayout`.

### A.2 Component tree
```
App
├── PairingScreen                 (!loggedIn)
│   ├── QrCanvas                  offline QR render of the ref string (qrcode lib)
│   ├── PairCodeForm              phone + optional custom code → auth_start_pair_code
│   └── ConnectionStatus
└── MainLayout                    (loggedIn) — CSS grid 340px | 1fr
    ├── ChatList
    │   └── ChatListItem          avatar/initials, name, last-msg preview, time, unread
    └── Conversation
        ├── (header)
        ├── MessageList           windowed, auto-scroll, loading state
        │   └── MessageBubble     fromMe styling, group sender, inline thumbnail
        └── MessageComposer       Enter to send / Shift+Enter newline
```

### A.3 IPC surface used (`lib/ipc.ts`)
Commands: `auth_status`, `auth_start_qr`, `auth_start_pair_code`, `connect`,
`disconnect`, `auth_logout`, `send_text`, `mark_read`, `get_contact`,
`get_profile_picture_url`.
Events (`wa://` envelope `{sessionId, kind, payload}`): `wa://auth/qr`,
`wa://auth/pair-code`, `wa://auth/paired`, `wa://auth/logged-out`,
`wa://conn/state`, `wa://message`, `wa://history`.

### A.4 Backend glue added for the frontend (`src-tauri`)
- **Normalized `MessageDto`** emitted on `wa://message` (id, chatJid, senderJid,
  fromMe, timestamp, pushName, text, `kind`, base64 `thumbnail`). Text uses
  `MessageExt::text_content()` → `get_caption()` so `extendedTextMessage` isn't
  lost.
- **`HistorySync` decode** → per-conversation `wa://history` (decoded off-thread
  via `HistorySyncStream::next_conversation()`), each carrying one `ChatDto` plus
  its `MessageDto`s — emitted per conversation for bounded memory / progressive
  render.
- **`get_contact` / `get_profile_picture_url`** commands for lazy name/avatar
  enrichment.
- **`normalize_chat_jid`** strips the device/agent suffix (`user.agent:device@server`
  → `user@server`) so phone-originated live messages and history land in one
  conversation.

### A.5 Event → store data flow (`lib/stores/events.ts`)
A single hub registers every `wa://` listener once and routes payloads:
- `wa://auth/*` and `wa://conn/state` → `session` store (+ `refreshStatus()`
  re-reads `auth_status`).
- `wa://history` → `upsertChatFromDto` per chat + `addHistoryMessages`.
- `wa://message` → `addMessage` + `touchChat` (bumps preview/time/unread) +
  `setChatName` from pushName.

Stores (`lib/stores/*.svelte.ts`, Svelte 5 runes + `SvelteMap`):
- **`session`** — loggedIn / connected / jid / qrCode / pairCode.
- **`chats`** — `Map<jid, Chat>`; `sortedChats()` by timestamp desc. Preview and
  time move together (`isNewer` guard) so the list never shows an old message's
  text beside a newer time.
- **`messages`** — `Map<jid, UiMessage[]>`, sorted asc, deduped by id.
  **Optimistic send**: `addOptimistic` appends a `status:"sending"` bubble;
  the echoed `wa://message` reconciles it (by id, or by matching a pending
  send's text) to `status:"sent"`.
- **`contacts`** — lazy `Map<jid, {name, pictureUrl}>`, fetched once per JID
  (success *and* failure cached). Name order: contact name → pushName → JID user.
  Deliberately **not** persisted (picture URLs expire).

### A.6 Frontend persistence (`lib/persist.ts`) — IndexedDB
Because `HistorySync` fires only at pairing, a plain relaunch has no events to
rebuild from. The event-derived projection is mirrored to the webview's
**IndexedDB** (`whatsvelte` db: `chats`, `messages`, `meta` stores) and
rehydrated on boot. No backend / SQLite change.

Key properties:
- **Stores every chat and every message — no cap.** RAM is bounded only on the
  render side (see A.7), never on storage.
- **Additive writes** (`put` per row, never `clear`) so an empty or
  partly-hydrated in-memory map can't wipe the cache. Only logout (`clearAll`)
  removes data.
- **Debounced** (600 ms) and **off until hydration completes**, so rehydrating
  doesn't immediately race a write back.
- **Account-keyed** (`accountJid` in `meta`): a different linked account
  invalidates the cache (compared via `normalizeJid` so device-suffix drift
  doesn't trigger a false wipe).
- **Hydration is independent of the racy `loggedIn` flag** (on restart the
  backend boots the session async, so `auth_status` briefly reports
  `loggedIn:false`). Chats missing from the `chats` store are reconstructed from
  their persisted messages (`ensureChat`), so the list self-heals.

### A.7 RAM: windowed rendering (`MessageList.svelte`)
Storage keeps everything; the DOM does not. The list mounts only the latest
`PAGE` (~40) messages and reveals older ones on scroll-up while preserving scroll
position. Bottom-pinned auto-follow keeps the newest message in view; a "Loading
messages…" state shows until a chat's history arrives.

### A.8 Layout / scroll correctness
Flex children default to `min-height:auto`, which defeats internal
`overflow-y:auto`. All scroll regions (chat list, message list) and the
`MainLayout` grid cells set `min-height:0` / `overflow:hidden` so lists scroll
internally and the composer stays on-screen.

### A.9 UI zoom (`lib/zoom.ts`)
`Ctrl/Cmd` + `+` / `-` / `0` (reset) scales the entire UI via CSS `zoom` on
`<html>`, clamped to **0.5×–2.0×** in 0.1 steps and persisted in `localStorage`.
The shortcuts are handled in-app (a global `keydown` listener installed from
`main.ts` before mount) rather than relying on the webview's native zoom, so
behaviour and persistence are identical across WebView2 and WKWebKit. `+` is
accepted as both `=` and `+` to cover shifted/unshifted layouts.

### A.10 Known gap carried into Part B
A single contact's `@lid` and `@s.whatsapp.net` identities are **not** merged
(different servers; needs the library's LID↔PN map) — they appear as two
conversations. Tracked under Part B "Identity & addressing".

---

## Part B — Rich object model  ✅ delivered

Part A treated every payload as text-or-thumbnail. Part B implements the real
WhatsApp object types and the interactions around them. Each maps to existing
library events/commands re-emitted through the same `wa://` envelope. Built in
milestones M1–M11, ordered easiest → hardest.

New IPC added in Part B:
- Commands: `send_reply`, `send_reaction`, `mark_read_messages`, `resolve_jid`,
  `download_media`, `set_chat_muted` / `set_chat_pinned` / `set_chat_archived`;
  `get_contact` extended with `verifiedName` + `lid`; `MessageDto` extended with
  `media` + `quoted`.
- Events: `wa://receipt` (normalized `ReceiptDto`), `wa://message/update`
  (`MessageUpdateDto` — revoke / edit / reaction), `wa://chat/flags`
  (`ChatFlagsDto` — mute / pin / archive).
- The `Event::Message` arm in `bridge.rs` now classifies **control messages**
  first (revoke / edit / reaction) and routes them to `wa://message/update`
  instead of adding a bubble; the message store exposes `applyReceipt` /
  `applyRevoke` / `applyEdit` / `applyReaction` patch fns.

### Identity & addressing
- **LID ↔ phone-number unification** (M10) — `resolve_jid` maps a contact's
  `@lid` identity to its `@s.whatsapp.net` form via the library's
  `get_lid_pn_entry`; the frontend then **merges** the two conversations (chat
  row + messages, in memory and in IndexedDB via `mergeChats`/`mergeMessages` +
  `deletePersisted`). Resolves the A.10 gap. Merge runs on new LID chats and on
  boot for cached ones.
- **Name addressing in chat** (M3) — display name resolves verified-business
  name → pushName → history `conv.name` → JID user; group participant names are
  resolved lazily through the contacts store in `MessageBubble`. **Library
  limitation:** there is no server getter for *saved* (address-book) contact
  names, so a contact saved only on the phone shows as pushName/number.

### Media & content types
- **Full media download** (M11) — image / video / audio / document fetched +
  decrypted by the backend (`download_media`, `client.download_from_params`) to a
  **content-addressed file** in `app_data_dir/media/`, returned as a path and
  rendered via Tauri's **asset protocol** (`convertFileSrc`) so bytes never cross
  the IPC/JS boundary. Images/stickers load eagerly; video/audio/docs are
  click-to-load. Requires the `protocol-asset` Cargo feature + an
  `assetProtocol` scope in `tauri.conf.json`.
- **Stickers** (M11) — received static & animated stickers render through the
  same media path. **Library limitation:** no API enumerates the user's
  *installed* sticker packs (only `fetch_sticker_pack(pack_id)` + received
  stickers), so a synced **sticker bar** is deferred.
- **Emoji** (M1, M8) — a dependency-free `EmojiPicker` in the composer
  (recent-emoji in `localStorage`); the same picker drives **reactions** from a
  per-bubble hover button (`send_reaction`, optimistic).

### Message lifecycle
- **Replies / quotes** (M12) — reply to a message from a per-bubble action; the
  composer shows a reply banner and `send_reply` attaches a quote `ContextInfo`
  (`build_quote_context` + `set_context_info`). Incoming/history replies carry
  the quoted preview in `MessageDto.quoted` (extracted from `ContextInfo`) and
  render as a quoted block inside the bubble (WhatsApp-style). The quoted preview
  is a best-effort text snapshot — the backend keeps no message store to
  reconstruct the original media proto.
- **Deleted messages** (M6) — `protocol_message` REVOKE → tombstone
  ("🚫 This message was deleted").
- **Edited messages** (M7) — `protocol_message` MESSAGE_EDIT → content replaced
  + an "edited" marker. **Follow-up:** the newer encrypted
  `secret_encrypted_message` edit path is not yet decoded.
- **Read receipts — display** (M4) — `Event::Receipt` normalized to
  `delivered`/`read`/`played`; per-message ticks (✓ / ✓✓ / blue ✓✓).
- **Read receipts — send** (M5) — an `IntersectionObserver` (`lib/receipts.ts`)
  acks incoming messages once actually scrolled into view, batched/debounced via
  `mark_read_messages`; gated on the Settings privacy toggle.

### App surfaces
- **Settings area** (M2) — `SettingsPanel` with Account (logout), Appearance
  (zoom, wallpaper), Chats (enter-to-send), Privacy (read-receipt toggle);
  device-local prefs in `localStorage` (`lib/stores/settings.svelte.ts`).
- **Server-synced chat settings** (M9) — mute / pin / archive via a chat
  context menu, optimistic with server echo on `wa://chat/flags`; pinned chats
  sort to the top.
- **Wallpapers** (M2) — per-chat + global conversation background (CSS color /
  gradient / image), applied on `Conversation` with a transparent `MessageList`.
- **Chat search** (M12) — a search box in the chat-list header filters chats by
  resolved name / JID / last-message preview (client-side over `sortedChats`).

### Implementation Module Summary Table

| # | Feature | Backend | Frontend |
| --- | --- | --- | --- |
| M1 | Emoji picker | — | EmojiPicker.svelte, lib/emoji.ts, composer caret-insert + recents |
| M2 | Settings + wallpapers | — | SettingsPanel.svelte, settings.svelte.ts, per-chat/global wallpaper, enter-to-send, privacy toggle |
| M3 | Name addressing | ContactDto + verifiedName/lid | group-participant name resolution via contacts store |
| M4 | Read receipts (display) | normalize Event::Receipt → wa://receipt | UiMessage.status, applyReceipt, ✓/✓✓/blue ticks |
| M5 | Read receipts (send) | mark_read_messages (per-message) | lib/receipts.ts IntersectionObserver, privacy-gated |
| M6 | Deleted messages | REVOKE → wa://message/update | applyRevoke tombstone |
| M7 | Edited messages | MESSAGE_EDIT → update | applyEdit + "edited" marker |
| M8 | Reactions | detect reaction_message + send_reaction | reaction chips + hover-to-react picker |
| M9 | Chat settings (mute/pin/archive) | chat_settings.rs + wa://chat/flags | context menu, pinned-first sort, badges |
| M10 | LID↔PN unification | resolve_jid via get_lid_pn_entry | mergeChats/mergeMessages + IndexedDB migration |
| M11 | Media + stickers | download_media → app-data file (asset protocol) | MessageMedia.svelte, lib/media.ts via convertFileSrc |

---

## Part C — Sending media 📤 ⏳ planned
Part B handled receiving/parsing media; the composer is still text-only. Part C
adds the send path — a clean mirror of the download path, since `whatsapp-rust`
already exposes `Client::upload()` + `media::*_message()` builders +
`send_message()`. The backend gains one `send_media` command; the frontend does
file selection, compression, thumbnailing, and recording. Built in three
incremental stages (C.1 → C.3), each independently shippable.

### C.1 Core send + images
- `send_media(jid, path, mediaType, options)` mirrors `download_media`: it reads
  bytes from a **path** (no base64-over-IPC bloat for large videos), `upload()`s,
  builds the proto via the matching `media::*_message()` builder, then
  `send_message()`s. Returns `SendResultDto`, exactly like `send_text`.
- **Path-based transfer**: picked files/documents pass their path directly (the
  dialog plugin returns paths); in-memory blobs (compressed images, clipboard
  pastes, recordings) are written to a temp file via the fs plugin first, and the
  frontend deletes that temp file once the send resolves.
- Image **compression** is canvas-based (standard: max edge ~1600px, JPEG q≈0.75;
  **HD** toggle keeps resolution at q≈0.9) — no heavy libraries. A small JPEG
  `jpegThumbnail` is generated for the inline preview.
- `AttachMenu` (📎) + `MediaPreview` (caption + HD toggle) in the composer, plus a
  composer **paste handler** for clipboard images.

### C.2 Recorder + audio/video
- `Recorder` uses `getUserMedia`/`MediaRecorder` for voice notes, video, and
  photo capture (camera frame → canvas snapshot); output flows through
  `MediaPreview`.
- **Video is sent as-is** + an auto-generated poster thumbnail (true transcoding
  is deferred — ffmpeg.wasm is ~30 MB and memory-heavy). Audio is sent with the
  recorder's actual mimetype and the `ptt` flag; opus transcode + waveform PCM
  are deferred (WKWebView emits `audio/mp4`, not `ogg/opus`).
- Needs webview camera/mic permission + macOS `Info.plist` usage strings
  (`NSCameraUsageDescription`, `NSMicrophoneUsageDescription`).

### C.3 Document viewing
- Received documents (txt / pdf / word / excel) **open in the OS default app** via
  the Tauri opener plugin — zero added bundle/footprint. In-app rendering of these
  formats is intentionally **not** done (pdf.js / mammoth / SheetJS are
  memory-heavy and would inflate the app's footprint).

---

## Build & verify
- `npm --prefix svelte-frontend run check` — svelte-check (0 errors).
- `npm --prefix svelte-frontend run build` — Vite production build.
- End-to-end (pairing, live send/receive, restart persistence) must be verified
  on a real linked account; the sandbox can't reach `web.whatsapp.com`.
