#!/bin/bash

# 快速重启 Kiro API 服务器

echo "🔄 重启 Kiro API 服务器..."
echo ""

# 查找并停止现有进程
if pgrep -f "aiclient2api-rust" > /dev/null; then
    echo "停止现有服务器..."
    pkill -f "aiclient2api-rust"
    sleep 1
fi

# 启动服务器
echo "启动服务器 (使用 INFO 日志级别)..."
echo "日志将同时输出到终端和 kiro.log 文件"
echo ""
echo "按 Ctrl+C 停止服务器"
echo "----------------------------------------"
echo ""

RUST_LOG=info ./target/release/aiclient2api-rust --config config.json 2>&1 | tee kiro.log

