#!/bin/bash

# 修复未使用的 imports

echo "修复 Go imports..."

# 检查哪些文件有问题
echo "检查可能的问题..."

# Kiro.go - bufio 可能未使用
if ! grep -q "bufio\." internal/adapter/kiro.go; then
    echo "⚠️  kiro.go 导入了 bufio 但未使用"
    # 移除 bufio import
    sed -i.bak '/"bufio"/d' internal/adapter/kiro.go
    echo "✅ 已移除 kiro.go 的 bufio import"
fi

# 创建一个简化版本，移除所有可能未使用的 imports
echo ""
echo "建议的修复方式："
echo "1. 如果您安装了 Go，运行: go mod tidy && gofmt -w ."
echo "2. 或者查看 GitHub Actions 的具体错误日志"
echo "3. 告诉我具体的错误信息，我来修复"


