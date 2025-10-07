#!/bin/bash

# 本地多平台构建脚本
# 用于在 GitHub Actions 不可用时本地构建所有平台版本

set -e

echo "=========================================="
echo "  AIClient-2-API 多平台构建"
echo "=========================================="
echo ""

# 检查 Go 是否安装
if ! command -v go &> /dev/null; then
    echo "❌ 错误: Go 未安装"
    echo "请先安装 Go: https://golang.org/dl/"
    exit 1
fi

echo "✅ Go 版本: $(go version)"
echo ""

# 创建构建目录
BUILD_DIR="build"
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR

echo "📦 正在下载依赖..."
go mod download
go mod tidy
echo "✅ 依赖下载完成"
echo ""

# 构建参数
VERSION=$(git describe --tags --always --dirty 2>/dev/null || echo "v0.9.0")
BUILD_TIME=$(date -u +%Y%m%d%H%M%S)
LDFLAGS="-s -w -X main.Version=$VERSION -X main.BuildTime=$BUILD_TIME"

echo "🔨 开始构建..."
echo "版本: $VERSION"
echo "构建时间: $BUILD_TIME"
echo ""

# 定义构建平台
declare -a PLATFORMS=(
    "linux:amd64"
    "linux:arm64"
    "darwin:amd64"
    "darwin:arm64"
    "windows:amd64"
    "windows:arm64"
)

# 构建每个平台
for platform in "${PLATFORMS[@]}"; do
    IFS=':' read -r GOOS GOARCH <<< "$platform"
    
    OUTPUT="aiclient2api-${GOOS}-${GOARCH}"
    if [ "$GOOS" = "windows" ]; then
        OUTPUT="${OUTPUT}.exe"
    fi
    
    echo "  📦 构建 ${GOOS}/${GOARCH}..."
    
    env GOOS=$GOOS GOARCH=$GOARCH CGO_ENABLED=0 go build \
        -v \
        -trimpath \
        -ldflags="$LDFLAGS" \
        -o "${BUILD_DIR}/${OUTPUT}" \
        . 2>&1 | grep -v "^#" || true
    
    if [ -f "${BUILD_DIR}/${OUTPUT}" ]; then
        SIZE=$(ls -lh "${BUILD_DIR}/${OUTPUT}" | awk '{print $5}')
        echo "  ✅ ${OUTPUT} (${SIZE})"
        
        # 打包
        cd $BUILD_DIR
        if [ "$GOOS" = "windows" ]; then
            zip -q "${OUTPUT%.exe}.zip" "$OUTPUT"
            echo "     → ${OUTPUT%.exe}.zip"
        else
            tar czf "${OUTPUT}.tar.gz" "$OUTPUT"
            echo "     → ${OUTPUT}.tar.gz"
        fi
        cd ..
    else
        echo "  ❌ 构建失败: ${OUTPUT}"
    fi
    echo ""
done

echo "=========================================="
echo "  构建完成！"
echo "=========================================="
echo ""
echo "📦 构建产物位于: ./$BUILD_DIR/"
ls -lh $BUILD_DIR/
echo ""
echo "🚀 使用方法:"
echo "  Linux:   tar xzf build/aiclient2api-linux-amd64.tar.gz"
echo "  macOS:   tar xzf build/aiclient2api-darwin-arm64.tar.gz"
echo "  Windows: 解压 build/aiclient2api-windows-amd64.zip"
echo ""

