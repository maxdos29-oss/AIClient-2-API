#!/bin/bash
# Build script for AIClient-2-API Rust version

set -e

echo "🦀 Building AIClient-2-API Rust Version..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✓ Rust version: $(rustc --version)"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Run tests
echo ""
echo "🧪 Running tests..."
cargo test --quiet

# Build in release mode
echo ""
echo "🔨 Building in release mode..."
cargo build --release

echo ""
echo "✅ Build completed successfully!"
echo ""
echo "📦 Binary location: ./target/release/aiclient2api-rust"
echo "📊 Binary size: $(du -h ./target/release/aiclient2api-rust | cut -f1)"
echo ""
echo "🚀 To run: ./target/release/aiclient2api-rust"
echo "📝 Or: cargo run --release"

