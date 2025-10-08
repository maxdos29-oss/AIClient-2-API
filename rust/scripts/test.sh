#!/bin/bash
# Test script for AIClient-2-API Rust version

set -e

echo "🧪 Running AIClient-2-API Rust Tests..."
echo ""

# Run all tests with output
echo "📋 Running all tests..."
cargo test -- --nocapture

echo ""
echo "📊 Running tests with coverage (if tarpaulin is installed)..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Stdout --output-dir ./coverage
else
    echo "ℹ️  cargo-tarpaulin not installed. Skipping coverage."
    echo "   Install with: cargo install cargo-tarpaulin"
fi

echo ""
echo "✅ All tests passed!"

