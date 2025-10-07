# Makefile for AIClient-2-API Rust version

.PHONY: help build test run clean docker fmt check clippy dev release install

help:
	@echo "AIClient-2-API Rust - Available commands:"
	@echo ""
	@echo "  make build      - Build the project in debug mode"
	@echo "  make release    - Build optimized release version"
	@echo "  make test       - Run all tests"
	@echo "  make run        - Run the server in debug mode"
	@echo "  make dev        - Run with auto-reload (requires cargo-watch)"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make fmt        - Format code with rustfmt"
	@echo "  make check      - Quick compile check"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make docker     - Build Docker image"
	@echo "  make install    - Install the binary to ~/.cargo/bin"
	@echo "  make help       - Show this help message"

build:
	@echo "🔨 Building project..."
	cargo build

release:
	@echo "🚀 Building release version..."
	cargo build --release
	@echo "✅ Binary: ./target/release/aiclient2api-rust"

test:
	@echo "🧪 Running tests..."
	cargo test -- --nocapture

run:
	@echo "▶️  Running server..."
	RUST_LOG=info cargo run

dev:
	@echo "🔧 Starting development mode..."
	@./scripts/dev.sh

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

fmt:
	@echo "✨ Formatting code..."
	cargo fmt

check:
	@echo "🔍 Checking code..."
	cargo check

clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

docker:
	@echo "🐳 Building Docker image..."
	@./scripts/docker-build.sh

install:
	@echo "📦 Installing binary..."
	cargo install --path .

# Development workflow
dev-full: fmt clippy test build
	@echo "✅ Development checks passed!"

# CI workflow
ci: fmt check clippy test
	@echo "✅ CI checks passed!"

# Quick development iteration
quick: fmt check
	@echo "✅ Quick checks passed!"

