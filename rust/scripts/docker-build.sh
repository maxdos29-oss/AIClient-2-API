#!/bin/bash
# Docker build script

set -e

echo "🐳 Building Docker image for AIClient-2-API Rust..."
echo ""

IMAGE_NAME="aiclient2api-rust"
VERSION="1.0.0"

echo "📦 Building image: ${IMAGE_NAME}:${VERSION}"
docker build -t ${IMAGE_NAME}:${VERSION} -t ${IMAGE_NAME}:latest .

echo ""
echo "✅ Docker image built successfully!"
echo ""
echo "📊 Image size:"
docker images ${IMAGE_NAME}:latest --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
echo ""
echo "🚀 To run:"
echo "   docker run -p 3000:3000 ${IMAGE_NAME}:latest"
echo ""
echo "   Or with docker-compose:"
echo "   docker-compose up -d"

