# whatsvelte-rust

A lightweight, low-RAM alternative to WhatsApp Desktop. Built with a Svelte frontend and a Rust backend, packaged together as a native desktop app using Tauri.

## Features

- **Rich Messaging**: Send and receive text, emojis, and media.
- **Cross-Platform**: Available for Windows, macOS, and Linux.
- **Offline-First Pairing**: Link your device via QR code or pair code.
- **Optimized UI**: Responsive two-pane chat interface with a built-in zoom feature.
- **Local History**: Chat history is persisted securely via IndexedDB.
- **Comprehensive Interactions**: Supports message replies, deleted message tombstones, edited messages, and read receipts.
- **Media Support**: Click-to-load videos, audios, documents, and stickers. Eager-loading for images.
- **Native OS Integration**: Received documents open directly in your OS's default applications.
- **Privacy Controls**: Read receipt toggles and chat settings (mute, pin, archive).

## Development Phases

### Phase 1: Backend Integration & IPC ✅ (Completed)
- Integrated the [`whatsapp-rust`](https://github.com/oxidezap/whatsapp-rust) library.
- Built a robust event-driven bridge (Tauri IPC) to pass messages between Rust and Svelte.
- Implemented the core session manager and authentication commands.
- *See [`docs/phase-1-api-surface.md`](docs/phase-1-api-surface.md) for detailed IPC API mapping.*

### Phase 2: Svelte Frontend ✅ (Completed)
- Developed a reactive SPA using Svelte 5, Vite, and TypeScript.
- Implemented the primary UI: pairing screens, chat list, and conversation view.
- Added comprehensive message lifecycle support (read receipts, edits, deletes).
- Implemented media downloading and local rendering via Tauri's asset protocol.
- Handled LID ↔ Phone Number identity unification.
- *See [`docs/phase-2-frontend.md`](docs/phase-2-frontend.md) for architectural data flow.*

### Phase 3: Media Sending ⏳ (Planned)
- Implement frontend file selection, compression, and thumbnail generation.
- Add voice recording via `MediaRecorder`.
- Enable document, video, and audio sending via the backend.

### Phase 4: Packaging & CI/CD ✅ (Completed)
- Cross-platform automated builds configured via GitHub Actions.
- Pushing a version tag (e.g., `v1.0.0`) automatically generates and publishes release binaries for Windows (`.msi`/`.exe`), macOS (`.dmg`/`.app`), and Linux (`.deb`/`.AppImage`).

## Quick start

```bash
make setup     # install the Tauri CLI + JS deps (once)
make server    # tauri dev — compiles the backend and opens the app
```

On first launch, the app boots the default WhatsApp session and shows a QR code to link a device (WhatsApp → Linked Devices), or use the pair-code field.

## Known Issues

- **macOS: "app is damaged and can't be opened"** — the `.dmg` is not yet
  notarized with an Apple Developer ID (the CI ships it ad-hoc signed). macOS
  Gatekeeper quarantines downloaded unsigned apps. After dragging the app to
  `/Applications`, clear the quarantine flag once:
  ```bash
  xattr -cr /Applications/whatsvelte-rust.app
  ```
  Then open it normally. (Alternatively: right-click the app → **Open** →
  **Open** on the warning dialog.) Windows and Linux builds are unaffected.
- **Number resolution is unreliable** — mapping a chat/sender to its real phone
  number (LID ↔ phone-number unification) does not always resolve correctly, so
  some contacts may show a raw JID/LID instead of a phone number or saved name.
- **Chat type resolution is unreliable** — distinguishing chat types (individual
  vs. group vs. newsletter/broadcast) is not fully reliable yet, so a chat may
  occasionally be classified or rendered as the wrong type.

## Documentation Reference
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System architecture overview.
- [`docs/phase-1-api-surface.md`](docs/phase-1-api-surface.md) - Backend IPC & Library Mapping.
- [`docs/phase-2-frontend.md`](docs/phase-2-frontend.md) - Frontend design and component structure.

## Notes

I know some people are against Webviews and Tauri in general, but this app has been made to achieve basic functionality of WhatsApp Web, which simply takes too much memory at idle, plus running it on Chromium based browsers is simply overkill. 

I noticed there was no directly available alternative without wrapping the same, high resource usage node environment web app in smaller and smaller Webviews, but the underlying bloat, memory leaks, lag all stayed the same, if not slightly better. So I made this to stay open as I do other resource intensive tasks, since a simple messaging app being responsible for taking up most of my usable, precious memory is simply annoying. I wanted to be able to stay connected without having to open my phone every time.

There is a way to make this better, without using Webviews or Tauri, by using the same whatsapp-rust backend but a Flutter frontend using flutter_to_rust_bridge, but it's an exponential increase in development effort for a slight increase in performance, so I might keep that for a future version, once I completely understand how to make the app work with the current stack. 

After some basic testing, I've come to the conclusion that this app takes upto 50 times less memory than WhatsApp, more rigorous testing to be done later. 

|                |Startup RAM|RAM Under Load<br>(Rapid Scrolling)|Idle RAM|
|----------------|-----------|--------------------------|--------|
|WhatsApp Desktop|1.6 GB     |2 GB                      |1 GB    |
|Whatsapp Web    |1 GB       |1.2 GB                    |> 600 MB|
|This App        |300 MB     |500 MB                    |< 100 MB|

On the release windows version on idle, this app takes as less as 20MB memory!

## Attribution
Backend library: [whatsapp-rust](https://github.com/oxidezap/whatsapp-rust)
