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

## Documentation Reference
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System architecture overview.
- [`docs/phase-1-api-surface.md`](docs/phase-1-api-surface.md) - Backend IPC & Library Mapping.
- [`docs/phase-2-frontend.md`](docs/phase-2-frontend.md) - Frontend design and component structure.

## Attribution
Backend library: [whatsapp-rust](https://github.com/oxidezap/whatsapp-rust)
