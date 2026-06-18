# whatsvelte-rust
Whatsapp integration made with Svelte and Rust backend, packaged with Tauri for a low RAM alternative to Whatsapp Desktop

## Architecture
Tauri monolith: `src-tauri/` is the single Rust backend, wrapping the
[`whatsapp-rust`](https://github.com/oxidezap/whatsapp-rust) library
(`rust-backend/`) and exposing it to a Svelte UI (`svelte-frontend/`) over Tauri
IPC — **commands** for actions, **events** for the live stream. No HTTP server.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and
[`docs/phase-1-api-surface.md`](docs/phase-1-api-surface.md).

## Quick start
```bash
make setup     # install the Tauri CLI + JS deps (once)
make server    # tauri dev — compiles the backend and opens the app
```
On first launch the app boots the default WhatsApp session and shows a QR code to
link a device (WhatsApp → Linked Devices), or use the pair-code field.

## Status
- **Phase 1 (backend integration & IPC) — done.** Session manager, event bridge,
  auth/messaging commands, dev harness UI. Builds and runs.
- Phase 2: real Svelte UI · Phase 3: testing · Phase 4: `tauri build` packaging.

## Attribution
Backend: [whatsapp-rust](https://github.com/oxidezap/whatsapp-rust)