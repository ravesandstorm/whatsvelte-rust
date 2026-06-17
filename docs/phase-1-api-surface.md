# Phase 1 — Library API surface & IPC mapping

> Maps the `whatsapp-rust` library API to the **Tauri Commands** and **Tauri
> Events** the Svelte frontend will use. See [`ARCHITECTURE.md`](./ARCHITECTURE.md)
> for the why. This is the IPC contract — the equivalent of an API reference,
> but for `invoke()`/`listen()` rather than HTTP routes.
>
> Conventions:
> - Command names are `snake_case` (Tauri default), grouped by `commands/*.rs`.
> - Event names are namespaced `wa://<area>/<thing>` to keep `listen()` keys tidy.
> - JIDs cross the IPC boundary as strings (`"15551234567@s.whatsapp.net"`),
>   parsed to `Jid` in Rust.
> - Every command returns `Result<T, ApiError>`; `ApiError` serializes to a
>   `{ code, message }` object the UI can switch on.

---

## A. How the bridge works

```text
whatsapp-rust Event bus ──register_handler(ChannelEventHandler)──▶ async_channel::Receiver<Arc<Event>>
                                                                          │  (bridge.rs task)
                                                                          ▼
                                                       match &*event { … } → app.emit("wa://…", payload)
```

- `Bot::builder().with_backend(SqliteStore::new("whatsapp.db")).build()` → `bot`.
- `bot.spawn()` → `BotHandle` (runs the client loop in the background); store it
  + `handle.client()` (`Arc<Client>`) in `AppState`.
- Register a `ChannelEventHandler` via `client.register_handler(...)`; a spawned
  task drains the receiver and re-emits each `Event` as a Tauri event.
- Commands borrow `Arc<Client>` from `AppState` (Tauri `State<…>`) and call the
  library directly.

`AppState` (sketch):

```rust
struct AppState {
    handle: tokio::sync::Mutex<Option<BotHandle>>, // lifecycle (logout/relogin)
    client: ArcSwapOption<Client>,                 // current Arc<Client>, or None
}
```

---

## B. Events (Rust → UI, via `app.emit`)

Source: `wacore::types::events::Event` (~47 kinds). Phase 1 forwards the subset
the UI needs first; the rest are added as Phase 2 demands them.

| Tauri event | Library `Event` variant | Payload (to JS) | Phase |
|-------------|-------------------------|-----------------|-------|
| `wa://auth/qr` | `PairingQrCode { code, timeout }` | `{ code, timeoutSecs }` | 1 |
| `wa://auth/pair-code` | `PairingCode { code, timeout }` | `{ code, timeoutSecs }` | 1 |
| `wa://auth/paired` | `PairSuccess` | `{ jid, pushName }` | 1 |
| `wa://auth/logged-out` | `LoggedOut` | `{ reason }` | 1 |
| `wa://conn/state` | `Connected` / `Disconnected` | `{ state: "connected"\|"disconnected" }` | 1 |
| `wa://message` | `Message(msg, info)` | normalized message DTO (see §E) | 1 |
| `wa://receipt` | `Receipt` | `{ chat, sender, ids, type, timestamp }` | 1 |
| `wa://presence` | `Presence` / `ChatPresence` | `{ jid, state, lastSeen? }` | 2 |
| `wa://chat/update` | `Pin/Mute/Archive/MarkRead/Delete…Update` | per-update DTO | 2 |
| `wa://group/update` | `GroupUpdate` | group delta DTO | 2 |
| `wa://history` | `HistorySync(LazyHistorySync)` | decoded conversation batch | 2 |
| `wa://contact/update` | `ContactUpdated` / `PushNameUpdate` | `{ jid, … }` | 2 |
| `wa://call` | `IncomingCall` | `{ from, callId, … }` | 3 |
| `wa://error/stream` | `StreamError` / `ConnectFailure` / `TemporaryBan` | `{ code, message }` | 2 |

> A catch-all `wa://event` (raw `EventKind` + JSON) can be emitted during dev so
> the throwaway UI can observe everything before DTOs are finalized.

---

## C. Commands (UI → Rust, via `invoke`)

### `commands/auth.rs` — lifecycle & pairing

| Command | Library call | Args → Returns | Phase |
|---------|--------------|----------------|-------|
| `auth_status` | `client.is_logged_in()` / `is_connected()` / `get_pn()` | `() → { loggedIn, connected, jid? }` | 1 |
| `auth_start_qr` | builder boots; QR arrives via `wa://auth/qr` | `() → ()` (idempotent; ensures client running) | 1 |
| `auth_start_pair_code` | `client.pair_with_code(PairCodeOptions{ phone, custom_code })` | `{ phone, customCode? } → { code }` | 1 |
| `auth_logout` | `client.logout()` | `() → ()` | 1 |
| `connect` | `client.connect()` | `() → ()` | 1 |
| `disconnect` | `client.disconnect()` | `() → ()` | 1 |

### `commands/messaging.rs` — send / edit / react

| Command | Library call | Args → Returns | Phase |
|---------|--------------|----------------|-------|
| `send_text` | `client.send_text(jid, text)` | `{ jid, text } → { messageId, to }` | 1 |
| `send_message` | `client.send_message(jid, wa::Message)` | `{ jid, message: MessageInput } → SendResult` | 1 |
| `reply_text` | `send_message` + quote context | `{ jid, text, quotedId, quotedFromMe } → SendResult` | 2 |
| `edit_message` | `client.edit_message(jid, id, msg)` | `{ jid, messageId, newText } → { messageId }` | 2 |
| `revoke_message` | `client.revoke_message(chat, id, RevokeType)` | `{ jid, messageId } → ()` | 2 |
| `send_reaction` | `client.send_reaction(key, emoji)` | `{ jid, messageId, fromMe, emoji } → SendResult` | 2 |
| `send_media` | `client.upload(...)` + `send_message` | `{ jid, kind, bytes\|path, caption? } → SendResult` | 2 |
| `mark_read` | `client.mark_chat_as_read(jid, from?)` | `{ jid, fromMessageId? } → ()` | 1 |
| `set_chat_state` | `client.chatstate().send_composing/paused(jid)` | `{ jid, state } → ()` | 2 |

### `commands/chats.rs` — chat actions & contacts

| Command | Library call | Args → Returns | Phase |
|---------|--------------|----------------|-------|
| `is_on_whatsapp` | `client.contacts().is_on_whatsapp(&[jid])` | `{ phones: [..] } → [{ jid, exists }]` | 2 |
| `get_user_info` | `client.contacts().get_user_info(&[jid])` | `{ jids } → { jid: UserInfo }` | 2 |
| `get_profile_picture` | `client.contacts().get_profile_picture(jid, hq)` | `{ jid, highQuality } → { url }` | 2 |
| `archive_chat` / `unarchive_chat` | `client.chat_actions().archive_chat(jid)` | `{ jid } → ()` | 2 |
| `pin_chat` / `unpin_chat` | `client.chat_actions().pin_chat(jid)` | `{ jid } → ()` | 2 |
| `mute_chat` / `unmute_chat` | `client.chat_actions().mute_chat(jid)` | `{ jid, untilMs? } → ()` | 2 |
| `delete_chat` / `clear_chat` | `client.chat_actions().delete_chat(jid, …)` | `{ jid } → ()` | 2 |
| `star_message` / `unstar_message` | `client.chat_actions().star_message(key)` | `{ jid, messageId, fromMe } → ()` | 2 |

### `commands/groups.rs` — groups (Phase 2+)

`create_group`, `get_group_info`, `list_groups`, `set_subject`,
`set_description`, `add_participants`, `remove_participants`,
`promote_participants`, `demote_participants`, `leave_group`, `get_invite_link`,
`join_with_invite_code` → thin wrappers over `client.groups().*`.

### `commands/presence.rs` / `commands/profile.rs` (Phase 2+)

`set_presence_available/unavailable`, `subscribe_presence`; `set_push_name`,
`set_status_text`, `set_profile_picture`.

> Newsletter / community / labels / polls / events are deferred until the UI
> needs them; each is a direct wrapper over the matching `client.<feature>()`
> module documented below.

---

## D. Full library reference (for wrapping)

Captured from the current `rust-backend` source so command authors don't have to
re-read it. Signatures abbreviated; file paths are where to confirm details.

### Bot / lifecycle — `src/bot.rs`, `src/client/lifecycle.rs`
- `Bot::builder() -> BotBuilder`; `.with_backend(SqliteStore)`,
  `.on_qr_code|on_pair_code|on_connected|on_logged_out|on_message|on_event(...)`,
  `.with_pair_code(PairCodeOptions)`, `.build().await -> Bot`.
- `bot.spawn() -> BotHandle`; `bot.run().await`; `bot.client() -> Arc<Client>`.
- `BotHandle::client()`, `::shutdown().await`, `::abort()`.
- `Client::connect/disconnect/reconnect/logout`, `is_connected`, `is_logged_in`,
  `wait_for_connected(timeout)`, `get_pn()/get_lid()/get_push_name()`,
  `register_handler(Arc<dyn EventHandler>)`.

### Messaging — `src/send.rs`, `src/message/`
- `send_message(to, wa::Message) -> SendResult`; `send_text(to, str)`;
  `forward_message`; `send_message_with_options(.., SendOptions)`.
- `revoke_message`, `keep_message`, `pin_message`/`unpin_message`.
- `SendResult { message_id, to }`, `SendResult::message_key()`.
- Message construction: `wa::Message::text(..)`, `MessageBuilderExt`/`MessageExt`
  helpers (`prelude`).

### Media — `src/upload.rs`, `src/download.rs`
- `upload(data, media_type, UploadOptions) -> UploadResponse`; `upload_stream`.
- `download(&dyn Downloadable) -> Vec<u8>`; `download_from_params`,
  `download_to_writer`.

### Pairing — `src/pair.rs`, `src/pair_code.rs`
- QR: `make_qr_data(...)`, surfaced via `PairingQrCode` event.
- Pair code: `client.pair_with_code(PairCodeOptions{ phone_number, custom_code,
  show_push_notification, .. }) -> String`.

### Feature modules — `client.<area>()` in `src/features/`
- **groups()** — query_info, get_participating, create_group, set_subject/description,
  add/remove/promote/demote_participants, leave, get_invite_link,
  join_with_invite_code, membership requests, profile pictures. (`groups.rs`)
- **contacts()** — is_on_whatsapp, get_profile_picture, get_user_info. (`contacts.rs`)
- **presence()** — set/set_available/set_unavailable, subscribe/unsubscribe. (`presence.rs`)
- **profile()** — set_status_text, set_push_name, set/remove_profile_picture. (`profile.rs`)
- **chat_actions()** — archive/pin/mute/star/mark_read/delete/clear/save_contact. (`chat_actions.rs`)
- **chatstate()** — send_composing/recording/paused. (`chatstate.rs`)
- **blocking()** — block/unblock/get_blocklist/is_blocked. (`blocking.rs`)
- **status()** — send_text/image/video/raw, revoke. (`status.rs`)
- **reaction** — send_reaction(key, emoji). (`reaction.rs`)
- **polls()** — create/create_quiz/vote/aggregate_votes. (`polls.rs`)
- **groups/community()** — create/link/unlink subgroups, get_subgroups. (`community.rs`)
- **newsletter()** — list_subscribed, get_metadata, create/join/leave, send_reaction,
  get_messages, subscribe_live_updates. (`newsletter.rs`)
- **labels()** — create/delete_label, add/remove_chat_label. (`labels.rs`)
- **events()** — create/respond (calendar-style events). (`events.rs`)
- **signal()** — encrypt/decrypt, validate_session, get_user_devices. (`signal.rs`)
- **media_reupload()**, **mex()**, **tctoken()**, **comments()** — advanced/rare.

### Events — `wacore/src/types/events.rs`
- `Event` enum (~47 variants), `Event::kind() -> EventKind`,
  `Event::as_message()`, `Event::message_text()`.
- `EventHandler { handle_event(Arc<Event>); interest() -> EventInterest }`.
- `ChannelEventHandler::new() -> (handler, Receiver<Arc<Event>>)` — **the bridge
  primitive**.

---

## E. DTOs that cross IPC (to design in scaffold)

These normalize protobuf/`wa::*` types into JS-friendly shapes so the frontend
never sees prost internals:

- `MessageDTO { id, chatJid, senderJid, fromMe, timestamp, kind, text?, media?,
  quoted?, reaction? }` — built from `(wa::Message, MessageInfo)`.
- `SendResultDTO { messageId, to }`.
- `ChatDTO`, `ContactDTO`, `GroupInfoDTO`, `UserInfoDTO` — added in Phase 2.
- `ApiError { code, message }` — `enum` of `NotLoggedIn | NotConnected |
  InvalidJid | Library(String)`.

---

## F. Phase 1 scaffold plan (build order)

Implemented only after this doc is approved.

1. **Create the Tauri app** in `src-tauri/` (target Tauri v2):
   - `Cargo.toml` deps: `tauri`, `whatsapp-rust = { path = "../rust-backend" }`,
     `tokio`, `serde`, `arc-swap`, `async-channel`, `log`/`env_logger`.
   - `tauri.conf.json` with `frontendDist`/`devUrl` pointing at the Svelte/Vite
     dev server; `build.rs`.
2. **`state.rs`** — `AppState` holding the `BotHandle` + current `Arc<Client>`;
   managed via `tauri::Builder::manage`.
3. **Boot the client** in `setup`: build the `Bot` with `SqliteStore`
   (DB path under the app-data dir), `bot.spawn()`, stash handle/client.
4. **`bridge.rs`** — register `ChannelEventHandler`, spawn a task that maps each
   `Event` → `app.emit("wa://…", dto)` for the Phase-1 event subset (+ dev
   catch-all `wa://event`).
5. **`error.rs`** — `ApiError` (`thiserror` + `serde::Serialize`).
6. **`commands/auth.rs` + `commands/messaging.rs`** — Phase-1 commands only:
   `auth_status`, `auth_start_pair_code`, `connect`, `disconnect`, `auth_logout`,
   `send_text`, `send_message`, `mark_read`. Register via `invoke_handler`.
7. **`MakeFile`** — `server: tauri dev`, `build: tauri build`.
8. **Smoke test** — `make server`; scan the emitted QR (logged + `wa://auth/qr`),
   confirm `wa://auth/paired` + `wa://conn/state`, send a text via `invoke` from
   the Tauri devtools console, and observe an inbound `wa://message`. No real UI
   required yet.

### Open items to settle during the scaffold
- **Tauri v1 vs v2** — plan assumes **v2** (current). Confirm before `init`.
- **DB location** — dev: `rust-backend/whatsapp.db`; prod: OS app-data dir.
- **Single session** assumed (one WhatsApp account per app install). Multi-account
  would add a `sessionId` arg to every command — out of scope unless requested.
- **Binary payloads** — media as base64 over IPC vs. Tauri asset protocol /
  temp-file paths (decide when `send_media`/downloads land in Phase 2).
