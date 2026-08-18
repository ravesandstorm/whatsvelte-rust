#!/usr/bin/env bash
set -euo pipefail

part="${1:-}"
if [[ "$part" != "major" && "$part" != "minor" && "$part" != "patch" ]]; then
    echo "usage: ./release.sh <major|minor|patch>" >&2
    exit 1
fi

echo "Building Dioxus Desktop release..."
cargo build --release
