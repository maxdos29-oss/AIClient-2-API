#!/bin/bash

# Kiro API 响应测试脚本
# 用于诊断 Cline "Invalid API Response" 错误

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  Kiro API Response Test${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""

# 检查参数
API_KEY="${1:-test-api-key}"
PORT="${2:-8080}"
BASE_URL="http://localhost:${PORT}"

echo -e "${BLUE}测试配置:${NC}"
echo "  - Base URL: ${BASE_URL}"
echo "  - API Key: ${API_KEY}"
echo ""

# 测试 1: 健康检查
echo -e "${YELLOW}[1/3] 测试健康检查...${NC}"
HEALTH_RESPONSE=$(curl -s "${BASE_URL}/health")
if echo "${HEALTH_RESPONSE}" | grep -q "healthy"; then
    echo -e "${GREEN}✓ 健康检查通过${NC}"
    echo "  ${HEALTH_RESPONSE}"
else
    echo -e "${RED}✗ 健康检查失败${NC}"
    echo "  ${HEALTH_RESPONSE}"
    exit 1
fi
echo ""

# 测试 2: 简单的 Claude API 调用
echo -e "${YELLOW}[2/3] 测试简单消息...${NC}"
RESPONSE=$(curl -s -X POST "${BASE_URL}/v1/messages" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${API_KEY}" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "messages": [
      {"role": "user", "content": "Say hello in one word"}
    ],
    "max_tokens": 50
  }')

echo "原始响应 (前 500 字符):"
echo "${RESPONSE}" | head -c 500
echo ""
echo ""

# 验证响应格式
echo -e "${YELLOW}[3/3] 验证响应格式...${NC}"

# 检查是否是有效的 JSON
if ! echo "${RESPONSE}" | jq . > /dev/null 2>&1; then
    echo -e "${RED}✗ 响应不是有效的 JSON${NC}"
    echo "完整响应:"
    echo "${RESPONSE}"
    exit 1
fi
echo -e "${GREEN}✓ 响应是有效的 JSON${NC}"

# 检查必需字段
REQUIRED_FIELDS=("id" "type" "role" "content" "model" "stop_reason" "usage")
for field in "${REQUIRED_FIELDS[@]}"; do
    if echo "${RESPONSE}" | jq -e ".${field}" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 字段 '${field}' 存在${NC}"
    else
        echo -e "${RED}✗ 缺少字段 '${field}'${NC}"
        exit 1
    fi
done

# 检查 type 字段
TYPE=$(echo "${RESPONSE}" | jq -r '.type')
if [ "${TYPE}" = "message" ]; then
    echo -e "${GREEN}✓ type = 'message'${NC}"
else
    echo -e "${RED}✗ type = '${TYPE}' (应该是 'message')${NC}"
fi

# 检查 role 字段
ROLE=$(echo "${RESPONSE}" | jq -r '.role')
if [ "${ROLE}" = "assistant" ]; then
    echo -e "${GREEN}✓ role = 'assistant'${NC}"
else
    echo -e "${RED}✗ role = '${ROLE}' (应该是 'assistant')${NC}"
fi

# 检查 content 数组
CONTENT_LENGTH=$(echo "${RESPONSE}" | jq '.content | length')
if [ "${CONTENT_LENGTH}" -gt 0 ]; then
    echo -e "${GREEN}✓ content 数组包含 ${CONTENT_LENGTH} 个块${NC}"
else
    echo -e "${RED}✗ content 数组为空${NC}"
    exit 1
fi

# 检查第一个 content block
FIRST_CONTENT_TYPE=$(echo "${RESPONSE}" | jq -r '.content[0].type')
if [ "${FIRST_CONTENT_TYPE}" = "text" ]; then
    TEXT_CONTENT=$(echo "${RESPONSE}" | jq -r '.content[0].text')
    TEXT_LENGTH=${#TEXT_CONTENT}
    echo -e "${GREEN}✓ 第一个 content block 类型是 'text'${NC}"
    echo -e "  文本长度: ${TEXT_LENGTH} 字符"
    echo -e "  文本内容: ${TEXT_CONTENT}"
elif [ "${FIRST_CONTENT_TYPE}" = "tool_use" ]; then
    TOOL_NAME=$(echo "${RESPONSE}" | jq -r '.content[0].name')
    echo -e "${GREEN}✓ 第一个 content block 类型是 'tool_use'${NC}"
    echo -e "  工具名称: ${TOOL_NAME}"
else
    echo -e "${RED}✗ 第一个 content block 类型是 '${FIRST_CONTENT_TYPE}' (应该是 'text' 或 'tool_use')${NC}"
    exit 1
fi

# 检查 usage 字段
INPUT_TOKENS=$(echo "${RESPONSE}" | jq -r '.usage.input_tokens')
OUTPUT_TOKENS=$(echo "${RESPONSE}" | jq -r '.usage.output_tokens')
echo -e "${GREEN}✓ usage.input_tokens = ${INPUT_TOKENS}${NC}"
echo -e "${GREEN}✓ usage.output_tokens = ${OUTPUT_TOKENS}${NC}"

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}  所有测试通过！响应格式正确${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""

# 显示完整的格式化响应
echo -e "${BLUE}完整格式化响应:${NC}"
echo "${RESPONSE}" | jq .

echo ""
echo -e "${YELLOW}提示:${NC}"
echo "如果 Cline 仍然报错，请检查："
echo "1. Cline 的 Base URL 设置是否正确 (${BASE_URL})"
echo "2. Cline 的 API Key 是否与配置文件中的一致"
echo "3. 查看服务器日志 (使用 RUST_LOG=debug)"
echo "4. 确认 Cline 使用的是 Claude API 模式"

