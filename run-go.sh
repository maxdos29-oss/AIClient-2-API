#!/bin/bash

# AIClient-2-API Go版本启动脚本

set -e

echo "==================================="
echo "  AIClient-2-API (Go Version)"
echo "==================================="
echo ""

# 检查Go是否安装
if ! command -v go &> /dev/null; then
    echo "❌ Go未安装。请先安装Go 1.21或更高版本。"
    echo ""
    echo "macOS: brew install go"
    echo "Ubuntu: sudo apt-get install golang-go"
    echo "或访问: https://golang.org/dl/"
    exit 1
fi

GO_VERSION=$(go version | awk '{print $3}')
echo "✓ 检测到 Go: $GO_VERSION"

# 检查配置文件
if [ ! -f "config.json" ]; then
    echo ""
    echo "⚠️  未找到 config.json，从示例文件复制..."
    if [ -f "config.json.example" ]; then
        cp config.json.example config.json
        echo "✓ 已创建 config.json，请根据需要编辑此文件"
    else
        echo "❌ config.json.example 也不存在"
        exit 1
    fi
fi

# 检查依赖
echo ""
echo "📦 检查依赖..."
if [ ! -d "vendor" ]; then
    echo "正在下载依赖..."
    go mod download
    go mod tidy
fi
echo "✓ 依赖检查完成"

# 构建或运行
echo ""
echo "🚀 启动选项："
echo "1) 直接运行（开发模式）"
echo "2) 构建并运行"
echo "3) 仅构建"
read -p "请选择 [1-3]: " choice

case $choice in
    1)
        echo ""
        echo "🏃 正在运行..."
        go run main.go "$@"
        ;;
    2)
        echo ""
        echo "🔨 正在构建..."
        go build -ldflags="-s -w" -o aiclient2api
        echo "✓ 构建完成: ./aiclient2api"
        echo ""
        echo "🏃 正在运行..."
        ./aiclient2api "$@"
        ;;
    3)
        echo ""
        echo "🔨 正在构建..."
        go build -ldflags="-s -w" -o aiclient2api
        echo "✓ 构建完成: ./aiclient2api"
        echo ""
        echo "运行命令: ./aiclient2api"
        ;;
    *)
        echo "无效选择"
        exit 1
        ;;
esac

