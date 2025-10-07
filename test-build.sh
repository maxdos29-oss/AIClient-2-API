#!/bin/bash

# 测试构建脚本 - 检查代码是否可以编译

set -e

echo "=========================================="
echo "  测试 Go 代码编译"
echo "=========================================="
echo ""

# 检查 Go 是否安装
if ! command -v go &> /dev/null; then
    echo "❌ Go 未安装"
    echo ""
    echo "安装方法:"
    echo "  macOS:   brew install go"
    echo "  Ubuntu:  sudo apt-get install golang-go"
    echo "  或访问:  https://golang.org/dl/"
    exit 1
fi

echo "✅ Go 版本: $(go version)"
echo ""

# 清理之前的构建
echo "🧹 清理旧文件..."
rm -f aiclient2api
rm -f go.sum

# 初始化模块
echo "📦 初始化 Go 模块..."
go mod tidy

# 检查语法错误
echo "🔍 检查语法..."
if ! go vet ./...; then
    echo "❌ 发现语法错误"
    exit 1
fi
echo "✅ 语法检查通过"
echo ""

# 检查格式
echo "🎨 检查代码格式..."
if [ -n "$(gofmt -l .)" ]; then
    echo "⚠️  代码格式需要调整:"
    gofmt -l .
    echo ""
    echo "自动格式化..."
    gofmt -w .
    echo "✅ 格式化完成"
else
    echo "✅ 代码格式正确"
fi
echo ""

# 尝试编译
echo "🔨 尝试编译..."
if go build -v -o aiclient2api .; then
    echo "✅ 编译成功！"
    echo ""
    ls -lh aiclient2api
    echo ""
    echo "🎉 构建测试通过！"
    echo ""
    echo "运行命令: ./aiclient2api"
    exit 0
else
    echo "❌ 编译失败"
    echo ""
    echo "请检查上面的错误信息"
    exit 1
fi
