#!/bin/bash
# Quick script to run with Kiro provider

set -e

echo "🦀 Starting AIClient-2-API Rust with Kiro Provider..."
echo ""

# Check if binary exists
if [ ! -f "./target/release/aiclient2api-rust" ]; then
    echo "📦 Binary not found. Building project..."
    cargo build --release
    echo "✅ Build complete!"
    echo ""
fi

echo "🚀 Starting server with Kiro provider..."
echo "   Host: 0.0.0.0"
echo "   Provider: claude-kiro-oauth"
echo "   Credentials: /Users/xuzhaokun/.aws/sso/cache/kiro-auth-token.json"
echo ""

# Run with Kiro configuration
RUST_LOG=info ./target/release/aiclient2api-rust \
  --host 0.0.0.0 \
  --model-provider claude-kiro-oauth \
  --kiro-oauth-creds-file /Users/xuzhaokun/.aws/sso/cache/kiro-auth-token.json

echo ""
echo "Server stopped."

