#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

_bump_version() {
    local part="$1"
    local conf
    
    # The app version lives in the Slint client's Cargo.toml
    local root
    root="$(git rev-parse --show-toplevel)" || return 1
    conf="$root/slint-client/Cargo.toml"

    # Check for uncommitted local changes (including untracked files)
    if [ -n "$(git status --porcelain)" ]; then
        echo "X Error: Working directory is not clean. Please commit or stash local changes before releasing."
        return 1
    fi

    # Ensure local and remote are in sync
    echo "Fetching latest remote state..."
    if ! git fetch -q; then
        echo "X Error: Failed to fetch from remote!"
        return 1
    fi

    local local_head
    local remote_head

    local_head=$(git rev-parse HEAD)
    
    # Safely get upstream tracking branch, catching the error if it doesn't exist
    if ! remote_head=$(git rev-parse @{u} 2>/dev/null); then
        echo "X Error: No upstream tracking branch configured for the current branch!"
        return 1
    fi

    if [ "$local_head" != "$remote_head" ]; then
        echo "X Error: Local and remote branches are out of sync. Please pull/push before releasing."
        return 1
    fi

    # Check if the file actually exists
    if [ ! -f "$conf" ]; then
        echo "X Error: $conf not found!"
        return 1
    fi

    local cur major minor patch new
    # First `version = "x.y.z"` line is the [package] version
    cur=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$conf" | head -n 1)
    IFS='.' read -r major minor patch <<< "$cur"

    # Increment the version based on input choice
    case "$part" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "usage: ./release.sh <major|minor|patch>"; return 1 ;;
    esac
    new="$major.$minor.$patch"

    # Rewrite only the first (package) version line, then refresh Cargo.lock's
    # entry for the app itself without touching any dependency.
    awk -v v="$new" '!done && /^version = "/ { print "version = \"" v "\""; done=1; next } { print }' \
        "$conf" > "$conf.tmp" && mv "$conf.tmp" "$conf" || return 1
    (cd "$root/slint-client" && cargo update --workspace --offline -q) || return 1

    # Git Operations
    git add "$conf" "$root/slint-client/Cargo.lock" && \
    git commit -m "chore: bump version to v$new" && \
    git tag "v$new" && \
    git push origin main && \
    git push origin "v$new" && \
    echo "Success! Released v$new"
}

# Accept terminal argument and trigger the core logic
if [ -z "$1" ]; then
    echo "usage: ./release.sh <major|minor|patch>"
    exit 1
fi

_bump_version "$1"
