# Common dev commands for whatsvelte-rust (Tauri monolith).
# Note: `make server` runs the whole app (Rust backend + WebView) via tauri dev. 

.PHONY: help setup server dev build frontend fmt clippy clean

help:
	@echo "Targets:"
	@echo "make setup	 - Install the Tauri CLI + JS deps (run once)"
	@echo "make server    	- Run the app in dev mode (tauri dev) — the continuous dev loop"
	@echo "make dev       	- Alias for 'server'"
	@echo "make build     	- Build the single-executable bundle (tauri build, Phase 4)"
	@echo "make frontend  	- Phase 1: static placeholder (no dev server). Phase 2: vite dev"
	@echo "make fmt       	- cargo fmt for the Tauri backend"
	@echo "make clippy    	- cargo clippy for the Tauri backend"
	@echo "make clean     	- Remove the Tauri target dir"

setup:
	npm install
	npm --prefix svelte-frontend install

# The "continuously running for development" loop: compiles the Rust backend,
# loads the frontend, and hot-reconnects on change.
server:
	npm run tauri dev

dev: server

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
