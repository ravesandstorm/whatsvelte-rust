# WhatDioxus Rust

A lightweight, pure Rust alternative to WhatsApp Desktop built with **Dioxus Desktop** and the `whatsapp-rust` protocol engine.

## Features
- **Pure Rust Stack**: Native Dioxus UI components and desktop rendering without JavaScript runtime overhead.
- **Zero Webview IPC Overhead**: UI state and protocol events run in pure Rust signals and Tokio channels.
- **Minimal Resource Footprint**: Fast startup and low memory usage.

## Development
- `make run` — Launch the Dioxus desktop application (`cargo run`).
- `make dev` — Run Dioxus dev mode with hot reload (`dx serve --platform desktop`).
- `make build` — Build release binary (`cargo build --release`).

## Architecture
```
├── src/               # Pure Rust Dioxus UI & App Handlers
├── rust-backend/      # Core WhatsApp Rust Protocol Library (Untouched)
├── Dioxus.toml        # Dioxus Application Configuration
└── Cargo.toml         # Pure Rust Workspace Manifest
```
