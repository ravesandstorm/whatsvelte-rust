# Whatsvelte-Rust — Architecture & Process

> Status: **planning** (Phase 1, pre-implementation). This document is the source
> of truth for *how* the project is structured and *why*. It supersedes the
> original "axum server" framing — see [Architecture decision](#architecture-decision).

## 1. Goal

Build a desktop application that mimics the WhatsApp Web client, on top of the
existing Rust WhatsApp-protocol implementation in [`rust-backend/`](../rust-backend),
and ship it as a **single executable** via [Tauri](https://tauri.app).

Three layers:

| Layer | Tech | Responsibility |
|-------|------|----------------|
| **Protocol core** | `rust-backend/` (`whatsapp-rust` library) | QR/pair auth, Signal E2E crypto, WebSocket transport to Meta, SQLite persistence, the full WhatsApp feature API |
| **Backend glue** | `src-tauri/` (new) | Owns a long-lived `Client`, exposes its methods as **Tauri Commands**, forwards the library's **event stream** to the UI as **Tauri Events** |
| **Frontend** | `svelte-frontend/` | WhatsApp-Web-style UI; calls `invoke()` for actions, `listen()` for live updates. No protocol logic. |

## 2. Architecture decision

The original brief described "an axum server that runs continuously" with the
frontend talking to it over HTTP. **We are not doing that.** We use the **Tauri
monolith + native IPC** model instead:

- **No axum / HTTP server.** Request/response goes over Tauri's IPC bridge via
  `#[tauri::command]` functions; the Svelte side calls `invoke('cmd', args)`.
- **No WebSocket/SSE for the UI.** Streaming (incoming messages, QR codes,
  receipts, presence) is pushed with Tauri Events: Rust calls
  `app.emit("event_name", payload)`, Svelte listens with
  `listen("event_name", …)`.
- **`tauri dev` is the "continuously running" dev loop.** It compiles the Rust
  backend, spins up the Vite/Svelte dev server, and hot-connects them. There is
  no separate process to babysit.

### Why

- One process, one binary — exactly what "ship as a singular executable" wants.
  No localhost port, no CORS, no auth layer between UI and backend, no second
  thing that can crash.
- Tauri's build tooling expects `tauri.conf.json` to sit next to the active
  `Cargo.toml`. Keeping the backend in `src-tauri` avoids fighting that later.
- The library already exposes a clean async `Client` API and an `Event` bus
  (`EventHandler` / `ChannelEventHandler`). Commands and Events map onto these
  almost 1:1 — an HTTP layer would be pure overhead.

### Trade-offs we accept

- The backend is only reachable from the bundled UI, not from external HTTP
  clients. That is fine for a desktop app; if a headless/remote mode is ever
  needed, an axum facade can be added later over the *same* command layer.
- Tauri Commands are the public contract instead of REST routes — so this repo
  documents **command + event signatures**, not OpenAPI paths.

### Refinement on "move everything into `src-tauri/src`"

A faithful monolith does **not** mean pasting the whole multi-crate
`rust-backend` workspace (`wacore`, `waproto`, transports, …) into
`src-tauri/src`. That workspace stays where it is. Instead:

- `src-tauri/Cargo.toml` depends on `whatsapp-rust` by **path**
  (`whatsapp-rust = { path = "../rust-backend" }`), and is the single source of
  truth for the *backend binary*.
- Our **integration glue** (command handlers, the event bridge, app state) lives
  in `src-tauri/src`.

This keeps the vendored library pristine and updatable while still giving Tauri
one `Cargo.toml` + `tauri.conf.json` to drive the build.

## 3. Target repository layout

```text
whatsvelte-rust/
├── docs/                       # this document + per-phase specs
├── rust-backend/               # vendored whatsapp-rust library workspace (unchanged)
├── src-tauri/                  # NEW — Tauri backend (the "monolith")
│   ├── Cargo.toml              #   depends on whatsapp-rust by path
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs             #   tauri::Builder, app setup
│       ├── state.rs            #   AppState: holds Arc<Client>/BotHandle
│       ├── bridge.rs           #   library Event stream -> app.emit(...)
│       ├── error.rs            #   command error type (-> JS-friendly)
│       └── commands/           #   #[tauri::command] fns grouped by area
│           ├── auth.rs
│           ├── messaging.rs
│           ├── chats.rs
│           └── ...
├── svelte-frontend/            # Svelte + Vite UI (Tauri's frontendDist)
└── MakeFile                    # dev shortcuts (to wrap `tauri dev` / `tauri build`)
```

## 4. Data & control flow

```text
        ┌─────────────────────────── Tauri process (single binary) ───────────────────────────┐
        │                                                                                       │
 Svelte UI  ──invoke('send_message')──▶  #[tauri::command]  ──▶  Client::send_message().await   │
 (WebView)  ◀──return value (Result)───   (commands/*.rs)                                        │
        │                                                                                       │
        │                                    AppState { BotHandle, Arc<Client> }                │
        │                                                  ▲                                     │
        │                                                  │ Client API                         │
        │                                       whatsapp-rust  ◀──WebSocket──▶  Meta servers     │
        │                                                  │                                     │
        │                                       Event bus (EventHandler)                         │
        │                                                  │                                     │
 Svelte UI  ◀──listen('wa://message')──   app.emit(...)  ◀─ bridge.rs (ChannelEventHandler)      │
        │                                                                                       │
        └───────────────────────────────────────────────────────────────────────────────────────┘
```

- **Commands** = synchronous-feeling RPC the UI initiates (send, mark read,
  create group, fetch info).
- **Events** = things the protocol initiates (incoming message, QR refresh,
  receipt, presence, connection state) — pushed without the UI asking.

## 5. The library surface we build on

`rust-backend` is a **library**, not a CLI. `src/main.rs` there is only a demo
bot. The pieces we wrap:

- **Lifecycle / auth** — `Bot::builder()` → `BotHandle` (background task);
  `Client::connect/disconnect/logout`, `is_connected`, `is_logged_in`. QR and
  pair codes arrive as events (`PairingQrCode`, `PairingCode`).
- **`Client` + feature extensions** — `client.send_message(...)`,
  `client.groups()`, `client.contacts()`, `client.presence()`,
  `client.chat_actions()`, etc. (~200 methods across 20 feature modules).
- **Event bus** — `Event` enum (~47 kinds) delivered through the `EventHandler`
  trait. `ChannelEventHandler` already adapts it to an `async_channel`, which is
  exactly what `bridge.rs` consumes to re-emit as Tauri events.
- **Persistence** — `SqliteStore` (`whatsapp.db`) keeps the session, so re-auth
  is only needed once.

Full method-by-method inventory and the command/event mapping live in
[`phase-1-api-surface.md`](./phase-1-api-surface.md).

## 6. Phased plan

The phases are revised for the Tauri-monolith model (Tauri is present from day
one, not bolted on at the end).

### Phase 1 — Backend integration & IPC contract  ← *we are here*
- Document the library API surface and map each operation to a **Tauri command**
  or **event** (this doc + `phase-1-api-surface.md`).
- Scaffold `src-tauri/`: depend on `whatsapp-rust`, boot a `BotHandle` in app
  state, bridge the event stream to `app.emit`, expose a first slice of commands
  (connect, get status, send message) and events (QR, message, connection).
- Outcome: `tauri dev` (or `make server`) launches, the app boots the client,
  surfaces a QR code to pair, and round-trips a sent/received message — verified
  with a throwaway UI/log before the real frontend exists.

### Phase 2 — Frontend  ✅ done
- Svelte 5 + Vite + TS SPA in `svelte-frontend/`, wired to Phase 1
  commands/events via `@tauri-apps/api`. `tauri dev` drives Vite via
  `beforeDevCommand` (devUrl `:5173`).
- Pairing screen with **offline** QR (the `qrcode` lib on a canvas) + pair-code
  form; two-pane WhatsApp layout (chat list, conversation, composer); message
  bubbles with fromMe distinction and inline image thumbnails; lazy contact
  names/avatars.
- Data is event-driven: the backend has no message DB, so the chat list/messages
  are rebuilt from `wa://history` (decoded from history-sync at pairing) + live
  `wa://message`. Optimistic send reconciles with the echoed event.
- **Frontend persistence** (`lib/persist.ts`): because HistorySync fires only
  once (at pairing), a plain relaunch has no events to rebuild from. The
  event-derived chats/messages are mirrored into the webview's **IndexedDB**
  (debounced) and rehydrated on boot — no backend/SQLite change. The cache is
  keyed to the linked account (`accountJid` meta) and wiped on logout or when a
  different account links. Avatars are deliberately not cached (profile-picture
  URLs expire; they re-fetch live). **Every** chat and message is stored (no
  cap) — RAM is bounded only on the render side (windowed list). Writes are
  additive (`put`, never `clear`) so a partly-hydrated map can't wipe the cache;
  hydration runs independent of the racy `loggedIn` flag and reconstructs any
  chat row missing from the chats store out of its persisted messages.
- Backend glue added: normalized `MessageDto` on `wa://message`, per-conversation
  `wa://history` decode, and `get_contact`/`get_profile_picture_url` commands.
- MVP hardening: chat JIDs are normalized (`normalize_chat_jid` strips the
  device/agent suffix) so phone-originated and history messages land in one
  conversation; the message list is windowed to the latest ~40 (loads older on
  scroll-up) to keep RAM flat on large chats; scroll regions get `min-height:0`
  so the composer stays on-screen and lists scroll internally with bottom-pinned
  auto-follow; chat previews track the newest message (preview + time move
  together). Known gap: LID↔phone-number addressing for one contact isn't merged.

### Phase 3 — Testing
- Rust: command-layer unit/integration tests; reuse the library's existing
  e2e/mock-server harness for protocol paths.
- Frontend: component tests + an end-to-end pass against a real linked test
  account.

### Phase 4 — Packaging
- `tauri build` → single signed executable per platform.
- Bundle config, app icons, auto-update story, DB location in the OS app-data
  dir.

## 7. Dev workflow (target)

```bash
make server     # → tauri dev  (compiles backend, runs Vite, hot-connects)
make frontend   # → vite dev   (UI-only iteration, if ever needed standalone)
make build      # → tauri build (Phase 4)
```

`MakeFile` currently only echoes placeholders; it will be filled in during the
Phase 1 scaffold.
