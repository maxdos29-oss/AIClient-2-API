#!/bin/bash
# Development script with auto-reload

set -e

echo "🔧 Starting development mode with auto-reload..."
echo ""

if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

echo "👀 Watching for changes..."
echo "Press Ctrl+C to stop"
echo ""

RUST_LOG=debug cargo watch -x 'run'

