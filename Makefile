# Common dev commands for whatdioxus-rust (Dioxus Desktop).

.PHONY: help run dev build fmt clippy clean

help:
	@echo "Targets:"
	@echo "  make run    - Run the Dioxus desktop app (cargo run)"
	@echo "  make dev    - Run the Dioxus desktop app with hot reload (dx serve)"
	@echo "  make build  - Build the release desktop application (cargo build --release)"
	@echo "  make fmt    - Format Rust code (cargo fmt)"
	@echo "  make clippy - Lint Rust code (cargo clippy)"
	@echo "  make clean  - Clean build artifacts (cargo clean)"

run:
	cargo run

dev:
	dx serve --platform desktop

build:
	cargo build --release

fmt:
	cargo fmt

clippy:
	cargo clippy

clean:
	cargo clean
