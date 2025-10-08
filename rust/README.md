# AIClient-2-API Rust Version 🦀

<div align="center">

**一个能将多种仅客户端内使用的大模型 API（Gemini CLI, Qwen Code Plus, Kiro Claude...），模拟请求，统一封装为本地 OpenAI 兼容接口的强大代理 - Rust 实现版本**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

[**中文**](#) | [**English**](#) | [**Node.js 版本**](../)

</div>

## 📖 简介

这是 AIClient-2-API 的 Rust 重写版本，完全保持与 Node.js 版本相同的功能和 API 兼容性，但具有以下优势：

- 🚀 **更高的性能**: Rust 的零成本抽象和编译时优化
- 🔒 **内存安全**: 无需垃圾回收的内存安全保证
- ⚡ **更快的启动**: 编译后的二进制文件，无需运行时环境
- 📦 **单一二进制**: 编译后可直接部署，无需安装依赖

## ✨ 核心功能

- ✅ **多模型统一接入**: 支持 Gemini、OpenAI、Claude、Kimi K2、GLM-4.5、Qwen Code 等
- ✅ **突破官方限制**: 通过 OAuth 授权突破官方免费 API 的速率和配额限制
- ✅ **OpenAI 兼容**: 完全兼容 OpenAI API 格式
- ✅ **账号池管理**: 支持多账号轮询、故障转移和配置降级
- ✅ **系统提示词管理**: 动态系统提示词注入
- ✅ **日志记录**: 完整的请求/响应日志功能

## 🔧 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/justlovemaki/AIClient-2-API.git
cd AIClient-2-API/rust

# 编译
cargo build --release

# 运行
./target/release/aiclient2api-rust
```

### 使用 Cargo

```bash
cargo install --path .
```

## 🚀 快速开始

### 1. 配置

创建 `config.json` 文件：

```json
{
  "host": "localhost",
  "port": 3000,
  "required_api_key": "your-secret-key",
  "model_provider": "gemini-cli-oauth",
  
  "gemini_oauth_creds_file_path": "~/.gemini/oauth_creds.json",
  "project_id": "your-gcp-project-id",
  
  "openai_api_key": "sk-...",
  "openai_base_url": "https://api.openai.com/v1",
  
  "claude_api_key": "sk-ant-...",
  "claude_base_url": "https://api.anthropic.com",
  
  "system_prompt_file_path": "input_system_prompt.txt",
  "system_prompt_mode": "overwrite",
  
  "prompt_log_mode": "file",
  "prompt_log_base_name": "prompt_log",
  
  "request_max_retries": 3,
  "request_base_delay": 1000,
  
  "cron_near_minutes": 15,
  "cron_refresh_token": true,
  
  "provider_pools_file_path": "provider_pools.json"
}
```

### 2. 运行

```bash
# 使用默认配置
./target/release/aiclient2api-rust

# 自定义配置文件
./target/release/aiclient2api-rust --config custom-config.json

# 设置日志级别
RUST_LOG=info ./target/release/aiclient2api-rust
```

### 3. 测试

```bash
# OpenAI 格式请求
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secret-key" \
  -d '{
    "model": "gemini-2.5-flash",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'

# Claude 格式请求
curl http://localhost:3000/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-secret-key" \
  -d '{
    "model": "claude-3-opus",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 1024
  }'
```

## 📚 API 端点

### OpenAI 兼容端点

- `POST /v1/chat/completions` - 聊天补全
- `GET /v1/models` - 列出可用模型

### Claude 兼容端点

- `POST /v1/messages` - 消息生成

### Gemini 兼容端点

- `GET /v1beta/models` - 列出模型
- `POST /v1beta/models/{model}:generateContent` - 生成内容
- `POST /v1beta/models/{model}:streamGenerateContent` - 流式生成

### 其他端点

- `GET /health` - 健康检查

### 路径切换提供商

可以通过路径前缀切换不同的提供商：

```bash
# 使用 Gemini
curl http://localhost:3000/gemini-cli-oauth/v1/chat/completions ...

# 使用 Claude
curl http://localhost:3000/claude-custom/v1/chat/completions ...

# 使用 OpenAI
curl http://localhost:3000/openai-custom/v1/chat/completions ...
```

## 🔐 认证

支持多种认证方式：

1. **Bearer Token**: `Authorization: Bearer <api-key>`
2. **API Key Header**: `x-api-key: <api-key>`
3. **Google API Key**: `x-goog-api-key: <api-key>`
4. **Query Parameter**: `?key=<api-key>`

## 🎯 账号池配置

创建 `provider_pools.json` 文件：

```json
{
  "gemini-cli-oauth": [
    {
      "uuid": "uuid-1",
      "GEMINI_OAUTH_CREDS_FILE_PATH": "./creds1.json",
      "PROJECT_ID": "project-1",
      "isHealthy": true,
      "errorCount": 0
    },
    {
      "uuid": "uuid-2",
      "GEMINI_OAUTH_CREDS_FILE_PATH": "./creds2.json",
      "PROJECT_ID": "project-2",
      "isHealthy": true,
      "errorCount": 0
    }
  ],
  "openai-custom": [
    {
      "uuid": "uuid-3",
      "OPENAI_API_KEY": "sk-...",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "isHealthy": true,
      "errorCount": 0
    }
  ]
}
```

## 🛠️ 开发

### 构建

```bash
# 开发模式
cargo build

# 发布模式（优化）
cargo build --release

# 检查代码
cargo check

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 项目结构

```
rust/
├── Cargo.toml              # 项目配置
├── src/
│   ├── main.rs            # 程序入口
│   ├── config.rs          # 配置管理
│   ├── server.rs          # HTTP 服务器
│   ├── common.rs          # 通用类型和工具
│   ├── adapter.rs         # 适配器接口
│   ├── convert.rs         # 格式转换
│   ├── pool_manager.rs    # 账号池管理
│   ├── strategies.rs      # 策略模式
│   └── providers/         # 提供商实现
│       ├── mod.rs
│       ├── gemini.rs
│       ├── openai.rs
│       ├── claude.rs
│       ├── kiro.rs
│       └── qwen.rs
└── README.md
```

## 🔄 与 Node.js 版本的差异

虽然 Rust 版本力求与 Node.js 版本功能对等，但在某些实现细节上会有所不同：

1. **性能**: Rust 版本通常有更好的性能和更低的内存占用
2. **类型安全**: Rust 的强类型系统提供编译时的更多保证
3. **错误处理**: 使用 Rust 的 `Result` 类型而非 JavaScript 的异常
4. **并发模型**: 使用 Tokio 异步运行时而非 Node.js 事件循环

## 📝 环境变量

```bash
# 日志级别
export RUST_LOG=info,aiclient2api_rust=debug

# 配置文件路径
export CONFIG_FILE=config.json

# HTTP 代理
export HTTP_PROXY=http://proxy:port
```

## 🐳 Docker 支持

```dockerfile
# Dockerfile 示例
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aiclient2api-rust /usr/local/bin/
EXPOSE 3000
CMD ["aiclient2api-rust"]
```

构建和运行：

```bash
docker build -t aiclient2api-rust .
docker run -p 3000:3000 -v $(pwd)/config.json:/app/config.json aiclient2api-rust
```

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📄 许可证

本项目采用 [GPL-3.0](https://www.gnu.org/licenses/gpl-3.0) 许可证。

## 🙏 致谢

- Node.js 原版项目
- Google Gemini CLI
- Anthropic Claude API
- OpenAI API

## ⚠️ 免责声明

本项目仅供学习和研究使用。用户需自行承担使用风险。请遵守各 AI 服务提供商的使用条款。

---

Made with ❤️ and 🦀 by the AIClient-2-API community

