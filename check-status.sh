#!/bin/bash

# 检查项目和 GitHub Actions 状态的脚本

echo "=========================================="
echo "  AIClient-2-API Go 版本状态检查"
echo "=========================================="
echo ""

# 1. 检查 Git 状态
echo "📋 Git 状态:"
echo "─────────────────────────────────────────"
echo "当前分支: $(git branch --show-current)"
echo "最新提交: $(git log -1 --oneline)"
echo "远程仓库: $(git remote get-url origin)"
echo ""

# 2. 检查标签
echo "🏷️  版本标签:"
echo "─────────────────────────────────────────"
git tag -l | sort -V | tail -5
echo ""

# 3. 检查文件统计
echo "📊 文件统计:"
echo "─────────────────────────────────────────"
echo "Go 源文件: $(find . -name '*.go' | wc -l | xargs)"
echo "文档文件: $(find . -name '*.md' | wc -l | xargs)"
echo "配置文件: $(find . -name '*.json' -o -name '*.yml' -o -name '*.yaml' | wc -l | xargs)"
echo ""

# 4. 代码行数
echo "📝 代码行数:"
echo "─────────────────────────────────────────"
if command -v wc &> /dev/null; then
    echo "Go 代码: $(find ./internal -name '*.go' 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}') 行"
    echo "main.go: $(wc -l main.go 2>/dev/null | awk '{print $1}') 行"
    echo "文档: $(find . -name '*-GO*.md' -o -name 'BUILD.md' -o -name 'MIGRATION.md' -o -name 'CONTRIBUTING.md' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}') 行"
fi
echo ""

# 5. Go 环境
echo "🔧 Go 环境:"
echo "─────────────────────────────────────────"
if command -v go &> /dev/null; then
    echo "✅ Go 已安装: $(go version | awk '{print $3}')"
    echo "GOPATH: $GOPATH"
    echo "GOOS: $(go env GOOS)"
    echo "GOARCH: $(go env GOARCH)"
else
    echo "❌ Go 未安装"
    echo "   安装: brew install go (macOS)"
    echo "   或访问: https://golang.org/dl/"
fi
echo ""

# 6. 构建测试
echo "🏗️  构建测试:"
echo "─────────────────────────────────────────"
if command -v go &> /dev/null; then
    if go build -o /tmp/aiclient2api-test . 2>&1 | head -5; then
        echo "✅ 构建成功"
        size=$(ls -lh /tmp/aiclient2api-test 2>/dev/null | awk '{print $5}')
        echo "二进制大小: $size"
        rm -f /tmp/aiclient2api-test
    else
        echo "❌ 构建失败，请检查错误信息"
    fi
else
    echo "⏭️  跳过 (Go 未安装)"
fi
echo ""

# 7. GitHub Actions 状态
echo "🤖 GitHub Actions:"
echo "─────────────────────────────────────────"
REPO_URL=$(git remote get-url origin | sed 's/\.git$//')
if [[ $REPO_URL == https://github.com/* ]]; then
    REPO_PATH=${REPO_URL#https://github.com/}
    echo "查看 Actions: https://github.com/$REPO_PATH/actions"
    echo "查看 Releases: https://github.com/$REPO_PATH/releases"
    echo "查看 v0.9.0: https://github.com/$REPO_PATH/releases/tag/v0.9.0"
else
    echo "仓库: $REPO_URL"
fi
echo ""

# 8. 快速命令
echo "⚡ 快速命令:"
echo "─────────────────────────────────────────"
echo "运行开发版本:  go run main.go"
echo "构建:         go build -o aiclient2api"
echo "运行构建版本:  ./aiclient2api"
echo "运行测试:      go test ./..."
echo "启动脚本:      ./run-go.sh"
echo ""

# 9. 重要链接
echo "🔗 重要链接:"
echo "─────────────────────────────────────────"
echo "快速入门: cat QUICKSTART-GO.md"
echo "构建指南: cat BUILD.md"
echo "迁移指南: cat MIGRATION.md"
echo "完成报告: cat PROJECT-COMPLETION-SUMMARY.md"
echo ""

# 10. 下一步建议
echo "🎯 建议的下一步:"
echo "─────────────────────────────────────────"
echo "1. 访问 GitHub Actions 查看构建状态"
echo "2. 等待构建完成 (~20-30 分钟)"
echo "3. 从 Releases 下载编译好的版本"
echo "4. 测试运行: ./aiclient2api"
echo "5. 阅读文档了解更多功能"
echo ""

echo "=========================================="
echo "  状态检查完成！"
echo "=========================================="

