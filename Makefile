# Common dev commands for whatsvelte-rust (Tauri monolith).
# Note: `make server` runs the whole app (Rust backend + WebView) via tauri dev. 

.PHONY: help setup server dev mock mock-web build frontend fmt clippy clean

help:
	@echo "Targets:"
	@echo "make setup	 			- Install the Tauri CLI + JS deps (run once)"
	@echo "make run    	 			- Run the app in dev mode (tauri dev) — the continuous dev loop"
	@echo "make dev       	 			- Alias for 'server'"
	@echo "make mock      	 			- The real Tauri app and webview on a MOCK backend (no WhatsApp connection)"
	@echo "make mock-web  	 			- Frontend only in a browser with mock data -> http://localhost:5174/?mock (NOT the real webview)"
	@echo "make build     	 			- Build the single-executable bundle (tauri build, Phase 4)"
	@echo "make frontend  	 			- Phase 1: static placeholder (no dev server). Phase 2: vite dev"
	@echo "make fmt       	 			- cargo fmt for the Tauri backend"
	@echo "make clippy    	 			- cargo clippy for the Tauri backend"
	@echo "make clean     	 			- Remove the Tauri target dir"
	@echo "make release-<major|minor|patch> 	- [RELEASE] PUSHES tag with release <major|minor|patch> bump"

setup:
	npm install
	npm --prefix svelte-frontend install

# The "continuously running for development" loop: compiles the Rust backend,
# loads the frontend, and hot-reconnects on change.
run:
	npm run tauri dev

dev: server

# The real app on fake data (src-tauri/src/mock.rs). Its own port means its own
# IndexedDB origin, so the real app's cached chats are left alone.
mock:
	WA_MOCK=1 npm run tauri dev -- --config src-tauri/tauri.mock.conf.json

# Browser-only frontend with fake chats/messages (svelte-frontend/src/lib/mock.ts).
# Scrolling and rendering differ from the real webview: prefer `make mock`.
mock-web:
	npm --prefix svelte-frontend run dev:mock

build:
	npm run tauri build

frontend:
	npm --prefix svelte-frontend run dev

fmt:
	cd src-tauri && cargo fmt

clippy:
	cd src-tauri && cargo clippy

clean:
	cd src-tauri && cargo clean

# Release commands
release-major:
	./release.sh major

release-minor:
	./release.sh minor

release-patch:
	./release.sh patch
