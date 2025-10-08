#!/bin/bash

# Kiro API 服务器启动脚本（调试模式）
# 启用详细日志以诊断问题

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Kiro API Server - Debug Mode${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# 检查配置文件
if [ ! -f "config.json" ]; then
    echo -e "${YELLOW}警告: config.json 不存在${NC}"
    echo "创建示例配置文件..."
    
    cat > config.json <<'EOF'
{
  "host": "127.0.0.1",
  "port": 8080,
  "model_provider": "claude-kiro-oauth",
  "required_api_key": "your-secret-key-here-change-this",
  "kiro": {
    "oauth_creds_file": "$HOME/.aws/sso/cache/kiro-auth-token.json"
  }
}
EOF
    
    echo -e "${GREEN}✓ 已创建 config.json${NC}"
    echo -e "${YELLOW}请编辑 config.json 并设置你的配置${NC}"
    exit 1
fi

# 检查是否已编译
if [ ! -f "target/release/aiclient2api-rust" ]; then
    echo -e "${YELLOW}程序未编译，正在编译...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ 编译完成${NC}"
    echo ""
fi

# 显示配置信息
echo -e "${BLUE}配置信息:${NC}"
echo "  - 配置文件: config.json"
echo "  - 日志级别: DEBUG (详细日志)"
echo "  - 端口: $(grep -o '"port": [0-9]*' config.json | cut -d' ' -f2)"
echo "  - Provider: $(grep -o '"model_provider": "[^"]*"' config.json | cut -d'"' -f4)"
echo ""

# 显示健康检查命令
PORT=$(grep -o '"port": [0-9]*' config.json | cut -d' ' -f2)
echo -e "${BLUE}测试命令:${NC}"
echo "  健康检查: curl http://localhost:${PORT}/health"
echo "  查看日志: tail -f kiro_debug.log"
echo ""

# 提示用户
echo -e "${YELLOW}提示:${NC}"
echo "  • 日志将输出到终端和 kiro_debug.log 文件"
echo "  • 按 Ctrl+C 停止服务器"
echo "  • 查看 KIRO_DEBUG_GUIDE.md 了解调试步骤"
echo ""

# 等待用户确认
echo -e "${GREEN}按 Enter 键启动服务器...${NC}"
read -r

echo -e "${GREEN}正在启动服务器...${NC}"
echo ""
echo -e "${BLUE}================================================${NC}"
echo ""

# 启动服务器并同时输出到终端和文件
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json 2>&1 | tee kiro_debug.log

