#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/target/release/disk-lens"

if [ -x "$BIN" ]; then
    exec "$BIN" "$@"
else
    if ! command -v cargo &> /dev/null; then
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        fi
    fi
    cargo build --release
    exec "$BIN" "$@"
fi
