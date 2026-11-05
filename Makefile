# Common dev commands for whatsvelte-rust (native Slint client, no WebView).
# The app lives in slint-client/; rust-backend/ is the vendored protocol library.

APP := slint-client

.PHONY: help setup run dev build package demo check test fmt clippy clean release-major release-minor release-patch

help:
	@echo "Targets:"
	@echo "make setup                           - Install cargo-packager (run once, for 'make package')"
	@echo "make run                             - Build and run the app (debug)"
	@echo "make dev                             - Alias for 'run'"
	@echo "make demo                            - Run with sample chats, no account/network (debug only)"
	@echo "                                       DEMO_OPEN=settings|reactions|delete|album|viewer|reply|suggest opens an overlay"
	@echo "make build                           - Optimized release binary"
	@echo "make package                         - Release installers (dmg/app, nsis, deb, AppImage)"
	@echo "make check                           - fmt check + clippy + tests (pre-commit gate)"
	@echo "make test                            - Unit tests for the client"
	@echo "make fmt                             - cargo fmt for the client"
	@echo "make clippy                          - cargo clippy for the client"
	@echo "make clean                           - Remove the client target dir"
	@echo "make release-<major|minor|patch>     - [RELEASE] PUSHES tag with release <major|minor|patch> bump"

setup:
	cargo install cargo-packager --locked

run:
	cd $(APP) && cargo run

dev: run

demo:
	cd $(APP) && WA_DEMO=$(or $(THEME),1) WA_DEMO_OPEN=$(DEMO_OPEN) cargo run

check:
	cd $(APP) && cargo fmt --check && cargo clippy -- -D warnings && cargo test

build:
	cd $(APP) && cargo build --release

package:
	cd $(APP) && cargo build --release && cargo packager --release

test:
	cd $(APP) && cargo test

fmt:
	cd $(APP) && cargo fmt

clippy:
	cd $(APP) && cargo clippy

clean:
	cd $(APP) && cargo clean

# Release commands
release-major:
	./release.sh major

release-minor:
	./release.sh minor

release-patch:
	./release.sh patch
