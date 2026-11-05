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
9. **Retire** — remove `src-tauri/`, `svelte-frontend/`, root npm files once the
   Slint client is at parity (separate, explicit step).

## Known gaps vs. the Svelte UI (deferred)

- Inline video/audio playback (opened externally instead).
- Voice-note recording (needs an audio capture crate, e.g. `cpal`).
- GIF search (Klipy) picker and animated sticker/GIF playback.
- Album grid / lightbox carousel (single-image viewer only).
