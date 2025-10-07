# 🚀 快速开始指南

## 1 分钟快速上手

### 第一步：安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 第二步：克隆和构建

```bash
cd /path/to/AIClient-2-API/rust
cargo build --release
```

### 第三步：配置

```bash
# 复制示例配置
cp config.example.json config.json

# 编辑配置文件，填入你的 API 密钥
nano config.json
```

最简配置（使用 OpenAI）：
```json
{
  "host": "localhost",
  "port": 3000,
  "required_api_key": "my-secret-key",
  "model_provider": "openai-custom",
  "openai_api_key": "sk-your-openai-key",
  "openai_base_url": "https://api.openai.com/v1"
}
```

### 第四步：运行

```bash
./target/release/aiclient2api-rust
```

看到这个输出说明成功了：
```
--- Unified API Server Configuration ---
  Host: localhost
  Port: 3000
  Primary Model Provider: openai-custom
------------------------------------------

Unified API Server running on http://localhost:3000
```

### 第五步：测试

```bash
# 打开新终端，测试 API
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer my-secret-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## 🎯 5 分钟高级配置

### 配置 Gemini (免费)

1. 获取 Gemini CLI 凭据：
   ```bash
   # 需要先安装 Gemini CLI
   # 然后运行授权
   ```

2. 配置文件：
   ```json
   {
     "model_provider": "gemini-cli-oauth",
     "gemini_oauth_creds_file_path": "~/.gemini/oauth_creds.json",
     "project_id": "your-gcp-project-id"
   }
   ```

### 配置 Claude

```json
{
  "model_provider": "claude-custom",
  "claude_api_key": "sk-ant-your-key",
  "claude_base_url": "https://api.anthropic.com"
}
```

### 配置多提供商（账号池）

创建 `provider_pools.json`：
```json
{
  "openai-custom": [
    {
      "uuid": "uuid-1",
      "OPENAI_API_KEY": "sk-key-1",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "isHealthy": true
    },
    {
      "uuid": "uuid-2",
      "OPENAI_API_KEY": "sk-key-2",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "isHealthy": true
    }
  ]
}
```

然后在 config.json 中：
```json
{
  "provider_pools_file_path": "provider_pools.json"
}
```

### 启用日志

```json
{
  "prompt_log_mode": "file",
  "prompt_log_base_name": "conversation_log"
}
```

### 自定义系统提示词

创建 `system_prompt.txt`：
```
You are a helpful, harmless, and honest AI assistant.
Always be polite and professional.
```

配置：
```json
{
  "system_prompt_file_path": "system_prompt.txt",
  "system_prompt_mode": "overwrite"
}
```

## 🔥 使用技巧

### 技巧 1: 路径切换提供商

无需修改配置，直接在 URL 中指定：

```bash
# 使用 Gemini
curl http://localhost:3000/gemini-cli-oauth/v1/chat/completions ...

# 使用 Claude
curl http://localhost:3000/claude-custom/v1/chat/completions ...
```

### 技巧 2: 开发模式

```bash
# 自动重载
make dev

# 或者
cargo watch -x run
```

### 技巧 3: 详细日志

```bash
RUST_LOG=debug cargo run
```

### 技巧 4: Docker 一键部署

```bash
docker-compose up -d
```

### 技巧 5: 健康检查

```bash
curl http://localhost:3000/health
```

## 🐛 故障排查

### 问题: 编译失败

```bash
# 更新 Rust
rustup update

# 清理重建
cargo clean
cargo build
```

### 问题: 测试失败

```bash
# 查看详细输出
cargo test -- --nocapture
```

### 问题: 连接失败

```bash
# 检查是否需要代理
export HTTP_PROXY=http://proxy:port
cargo run
```

### 问题: 权限错误

```bash
# 给脚本执行权限
chmod +x scripts/*.sh
```

## 📱 客户端集成

### LobeChat

设置 API 地址：
```
http://localhost:3000
```

API Key:
```
my-secret-key
```

### Cursor / VS Code

在设置中配置：
```json
{
  "openai.api.baseUrl": "http://localhost:3000",
  "openai.api.key": "my-secret-key"
}
```

### 命令行

```bash
# 使用 curl
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer my-secret-key" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'
```

## 📚 下一步阅读

- 📖 [README.md](./README.md) - 完整文档
- 🏗️ [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构设计
- 🔧 [BUILD_AND_RUN.md](./BUILD_AND_RUN.md) - 详细构建指南
- ✨ [FEATURES_IMPLEMENTED.md](./FEATURES_IMPLEMENTED.md) - 功能清单

---

**需要帮助？** 查看文档或提交 Issue！

**准备好了？** 开始构建你的 AI 应用吧！🚀

