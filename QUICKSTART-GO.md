# Go 版本快速入门

5 分钟快速上手 AIClient-2-API Go 版本。

## 🚀 快速开始

### 前提条件

确保已安装 Go 1.21+:

```bash
go version
# 应该输出: go version go1.21.x ...
```

如果没有安装，请访问: https://golang.org/dl/

### 三步启动

#### 1️⃣ 克隆项目

```bash
git clone https://github.com/justlovemaki/AIClient-2-API.git
cd AIClient-2-API
```

#### 2️⃣ 配置

```bash
# 复制示例配置
cp config.json.example config.json

# 编辑配置文件，设置你的 API 密钥
# 至少需要配置一个提供商的密钥
nano config.json
```

最小配置示例（使用 OpenAI）:

```json
{
  "REQUIRED_API_KEY": "your-custom-api-key-here",
  "SERVER_PORT": 3000,
  "HOST": "localhost",
  "MODEL_PROVIDER": "openai-custom",
  "OPENAI_API_KEY": "sk-your-openai-key-here",
  "OPENAI_BASE_URL": "https://api.openai.com/v1"
}
```

#### 3️⃣ 运行

```bash
# 使用启动脚本（推荐）
./run-go.sh

# 或手动运行
go run main.go
```

服务启动后会显示:

```
[Server] Starting Unified API Server on http://localhost:3000

Supports multiple API formats:
  • OpenAI-compatible: /v1/chat/completions, /v1/models
  • Gemini-compatible: /v1beta/models
  • Claude-compatible: /v1/messages
  • Health check: /health
```

## ✅ 验证安装

### 测试健康检查

```bash
curl http://localhost:3000/health
```

预期响应:

```json
{
  "status": "healthy",
  "timestamp": "2025-10-07T...",
  "provider": "openai-custom"
}
```

### 测试 Chat Completions

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer your-custom-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Say hello!"}
    ]
  }'
```

### 测试模型列表

```bash
curl http://localhost:3000/v1/models \
  -H "Authorization: Bearer your-custom-api-key-here"
```

## 🎯 常用场景

### 场景 1: 使用 OpenAI

```bash
# 配置 config.json
{
  "MODEL_PROVIDER": "openai-custom",
  "OPENAI_API_KEY": "sk-xxx",
  "OPENAI_BASE_URL": "https://api.openai.com/v1"
}

# 启动
go run main.go

# 或使用命令行参数
go run main.go \
  --model-provider openai-custom \
  --openai-api-key sk-xxx \
  --openai-base-url https://api.openai.com/v1
```

### 场景 2: 使用 Claude

```bash
# 配置 config.json
{
  "MODEL_PROVIDER": "claude-custom",
  "CLAUDE_API_KEY": "sk-ant-xxx",
  "CLAUDE_BASE_URL": "https://api.anthropic.com"
}

# 启动
go run main.go

# 使用 Claude Messages API
curl -X POST http://localhost:3000/v1/messages \
  -H "x-api-key: your-custom-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-7-sonnet-20250219",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### 场景 3: 使用 Gemini (需要 OAuth)

```bash
# 1. 获取 Gemini OAuth 凭据
#    参考: https://ai.google.dev/gemini-api/docs/oauth

# 2. 配置
{
  "MODEL_PROVIDER": "gemini-cli-oauth",
  "GEMINI_OAUTH_CREDS_FILE_PATH": "~/.gemini/oauth_creds.json",
  "PROJECT_ID": "your-gcp-project-id"
}

# 3. 启动
go run main.go

# 4. 使用
curl -X POST http://localhost:3000/v1beta/models/gemini-2.5-flash:generateContent \
  -H "x-goog-api-key: your-custom-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{"parts": [{"text": "Hello"}]}]
  }'
```

### 场景 4: 使用账号池（多账号负载均衡）

```bash
# 1. 配置 provider_pools.json
{
  "openai-custom": [
    {
      "OPENAI_API_KEY": "sk-key1",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "uuid": "account-1"
    },
    {
      "OPENAI_API_KEY": "sk-key2",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "uuid": "account-2"
    }
  ]
}

# 2. 在 config.json 中启用
{
  "PROVIDER_POOLS_FILE_PATH": "provider_pools.json"
}

# 3. 启动（会自动轮询使用两个账号）
go run main.go
```

## 🐳 Docker 快速启动

### 构建镜像

```bash
docker build -f Dockerfile.golang -t aiclient2api:go .
```

### 运行容器

```bash
docker run -d \
  --name aiclient2api \
  -p 3000:3000 \
  -v $(pwd)/config.json:/root/config.json \
  aiclient2api:go
```

### 使用 Docker Compose

创建 `docker-compose.yml`:

```yaml
version: '3.8'
services:
  aiclient2api:
    build:
      context: .
      dockerfile: Dockerfile.golang
    ports:
      - "3000:3000"
    volumes:
      - ./config.json:/root/config.json
    restart: unless-stopped
```

运行:

```bash
docker-compose up -d
```

## 🔧 配置选项

### 最小配置

```json
{
  "REQUIRED_API_KEY": "123456",
  "MODEL_PROVIDER": "openai-custom",
  "OPENAI_API_KEY": "sk-xxx"
}
```

### 完整配置

```json
{
  "REQUIRED_API_KEY": "your-api-key",
  "SERVER_PORT": 3000,
  "HOST": "localhost",
  "MODEL_PROVIDER": "openai-custom",
  
  "OPENAI_API_KEY": "sk-xxx",
  "OPENAI_BASE_URL": "https://api.openai.com/v1",
  
  "CLAUDE_API_KEY": "sk-ant-xxx",
  "CLAUDE_BASE_URL": "https://api.anthropic.com",
  
  "GEMINI_OAUTH_CREDS_FILE_PATH": "~/.gemini/oauth_creds.json",
  "PROJECT_ID": "your-gcp-project",
  
  "SYSTEM_PROMPT_FILE_PATH": "input_system_prompt.txt",
  "SYSTEM_PROMPT_MODE": "overwrite",
  
  "PROMPT_LOG_MODE": "none",
  "PROMPT_LOG_BASE_NAME": "prompt_log",
  
  "REQUEST_MAX_RETRIES": 3,
  "REQUEST_BASE_DELAY": 1000,
  
  "CRON_NEAR_MINUTES": 15,
  "CRON_REFRESH_TOKEN": true,
  
  "PROVIDER_POOLS_FILE_PATH": "provider_pools.json"
}
```

## 📱 客户端集成

### JavaScript/TypeScript

```javascript
const response = await fetch('http://localhost:3000/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer your-api-key',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'gpt-3.5-turbo',
    messages: [{role: 'user', content: 'Hello'}]
  })
});

const data = await response.json();
console.log(data);
```

### Python

```python
import requests

response = requests.post(
    'http://localhost:3000/v1/chat/completions',
    headers={'Authorization': 'Bearer your-api-key'},
    json={
        'model': 'gpt-3.5-turbo',
        'messages': [{'role': 'user', 'content': 'Hello'}]
    }
)

print(response.json())
```

### OpenAI SDK

```python
from openai import OpenAI

client = OpenAI(
    api_key="your-api-key",
    base_url="http://localhost:3000/v1"
)

response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[{"role": "user", "content": "Hello"}]
)

print(response.choices[0].message.content)
```

## 🛠️ 开发模式

### 启用日志

```bash
go run main.go --log-prompts console
```

### 使用自定义端口

```bash
go run main.go --port 8080
```

### 多个参数组合

```bash
go run main.go \
  --port 8080 \
  --api-key my-secret-key \
  --model-provider openai-custom \
  --openai-api-key sk-xxx \
  --log-prompts console
```

## ❓ 常见问题

### Q: 如何切换不同的提供商？

A: 通过三种方式:

1. 配置文件: 修改 `MODEL_PROVIDER`
2. 命令行: `--model-provider gemini-cli-oauth`
3. 请求头: `Model-Provider: claude-custom`

### Q: 如何启用日志？

A: 设置 `--log-prompts console` 或在 config.json 中设置 `"PROMPT_LOG_MODE": "console"`

### Q: 如何使用多个账号？

A: 配置 `provider_pools.json`，系统会自动轮询使用

### Q: 如何验证服务是否正常？

A: 访问健康检查端点: `curl http://localhost:3000/health`

### Q: 编译后的二进制文件在哪？

A: 运行 `go build` 后会生成 `aiclient2api` 文件

## 📚 进阶学习

- 详细文档: [README-GO.md](./README-GO.md)
- 构建指南: [BUILD.md](./BUILD.md)
- 迁移指南: [MIGRATION.md](./MIGRATION.md)
- 开发状态: [GO-VERSION-STATUS.md](./GO-VERSION-STATUS.md)

## 🆘 获取帮助

- GitHub Issues: https://github.com/justlovemaki/AIClient-2-API/issues
- 查看日志: 启用 `--log-prompts console` 查看详细输出
- 检查配置: 确保所有必需的字段都已设置

## 🎉 完成！

现在你已经成功运行 AIClient-2-API Go 版本了！

下一步:
1. ✅ 测试不同的 API 端点
2. ✅ 集成到你的应用中
3. ✅ 配置多个提供商
4. ✅ 使用账号池提高可用性
5. ✅ 阅读完整文档了解更多功能

祝使用愉快！🚀

