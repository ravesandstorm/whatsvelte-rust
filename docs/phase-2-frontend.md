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

## Part B — Rich object model  ⏳ planned

Part A treats every payload as text-or-thumbnail. Part B implements the real
WhatsApp object types and the interactions around them. Each maps to existing
library events/commands re-emitted through the same `wa://` envelope; the
IndexedDB schema gains stores/fields per object type as they land.

### Identity & addressing
- **LID ↔ phone-number unification** — merge a contact's `@lid` and
  `@s.whatsapp.net` identities into one conversation using the library's LID↔PN
  mapping. (Resolves the A.10 gap.)
- **Name addressing in chat** — show the saved contact / business / pushName for
  incoming messages and group participants instead of the raw JID.

### Media & content types
- **Full media download** — image / video / audio / document beyond the inline
  `jpegThumbnail`, fetched on demand and cached locally.
- **Stickers** — render static & animated stickers; a **sticker bar** in the
  composer populated from the user's sticker packs (synced from
  history / app-state objects).
- **Emoji bar** — emoji picker in the composer; emoji **reactions** on messages.

### Message lifecycle
- **Deleted messages** — render "this message was deleted" from revoke events.
- **Edited messages** — show edited content with an "edited" marker.
- **Read receipts (display)** — per-message sent / delivered / read ticks.
- **Read receipts (send)** — emit read events for messages actually **rendered
  on-screen** (viewport-driven), not merely on chat open.

### App surfaces
- **Settings area** — account, notifications, privacy, theme.
- **Wallpapers** — per-chat / global conversation background.

---

## Build & verify
- `npm --prefix svelte-frontend run check` — svelte-check (0 errors).
- `npm --prefix svelte-frontend run build` — Vite production build.
- End-to-end (pairing, live send/receive, restart persistence) must be verified
  on a real linked account; the sandbox can't reach `web.whatsapp.com`.
