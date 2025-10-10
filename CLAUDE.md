# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概述

AIClient-2-API 是一个强大的代理工具，可将多种 AI 客户端 API（Gemini CLI、Qwen Code Plus、Kiro Claude）转换为 OpenAI 兼容的端点。项目有**两个实现版本**：

1. **Node.js 版本**（根目录）- 原始实现
2. **Rust 版本**（[rust/](rust/)）- 高性能重写版本，性能提升 3-4 倍

两个版本**完全 API 兼容**，实现相同的功能。

## 架构设计

### 核心设计模式

- **策略模式**：不同的 AI 提供商（Gemini、OpenAI、Claude、Kiro、Qwen）实现统一接口
- **适配器模式**：在不同 API 格式之间转换（OpenAI ↔ Claude ↔ Gemini）
- **提供商池**：多账号管理，支持轮询、故障转移和降级

### 核心组件

**Node.js 版本：**
- [src/api-server.js](src/api-server.js) - 主 HTTP 服务器
- [src/adapter.js](src/adapter.js) - 提供商适配器工厂
- [src/convert.js](src/convert.js) - 格式转换逻辑
- [src/provider-strategy.js](src/provider-strategy.js) - 策略实现
- [src/provider-pool-manager.js](src/provider-pool-manager.js) - 账号池管理
- 提供商实现：[src/gemini/](src/gemini/)、[src/claude/](src/claude/)、[src/openai/](src/openai/)

**Rust 版本：**
- [rust/src/main.rs](rust/src/main.rs) - 入口点
- [rust/src/server.rs](rust/src/server.rs) - Axum HTTP 服务器
- [rust/src/adapter.rs](rust/src/adapter.rs) - 提供商适配器
- [rust/src/convert.rs](rust/src/convert.rs) - 格式转换
- [rust/src/strategies.rs](rust/src/strategies.rs) - 策略实现
- [rust/src/pool_manager.rs](rust/src/pool_manager.rs) - 账号池
- [rust/src/providers/](rust/src/providers/) - 提供商实现（gemini.rs、claude.rs、openai.rs、kiro.rs、qwen.rs）

### 提供商系统

每个提供商实现统一接口，包含以下方法：
- `generate_content()` - 非流式生成
- `stream_generate_content()` - 流式生成
- `list_models()` - 列出可用模型

支持的提供商：
- `gemini-cli-oauth` - Gemini OAuth 方式（绕过 API 限制）
- `openai-custom` - OpenAI 兼容 API
- `claude-custom` - Claude 兼容 API
- `claude-kiro-oauth` - 通过 Kiro 客户端使用 Claude（免费使用 Sonnet 4）
- `openai-qwen-oauth` - Qwen Code Plus OAuth 方式

## 常用开发任务

### Node.js 版本

**运行服务器：**
```bash
node src/api-server.js
# 使用自定义配置：
node src/api-server.js --port 8080 --api-key my-key
```

**运行测试：**
```bash
npm test                    # 所有测试
npm run test:unit          # 仅单元测试
npm run test:integration   # 仅集成测试
npm run test:coverage      # 带覆盖率报告
```

**关键启动参数：**
- `--host <地址>` - 服务器地址（默认：localhost）
- `--port <端口>` - 服务器端口（默认：3000）
- `--api-key <密钥>` - 身份验证 API 密钥
- `--model-provider <提供商>` - 使用的 AI 提供商
- `--log-prompts <模式>` - 日志模式：console、file 或 none
- `--system-prompt-file <路径>` - 系统提示词文件
- `--system-prompt-mode <模式>` - overwrite（覆盖）或 append（追加）

### Rust 版本

**构建和运行：**
```bash
cd rust
cargo build --release                    # 生产构建
cargo run                                # 调试模式
RUST_LOG=debug cargo run                 # 带调试日志
./target/release/aiclient2api-rust       # 运行编译后的二进制文件
```

**使用 Makefile：**
```bash
cd rust
make build      # 调试构建
make release    # 发布构建
make test       # 运行测试
make run        # 调试模式运行
make fmt        # 格式化代码
make clippy     # 运行 linter
```

**运行测试：**
```bash
cd rust
cargo test                              # 所有测试
cargo test --test integration_tests    # 集成测试
cargo test --test provider_tests       # 提供商测试
cargo test -- --nocapture              # 显示输出
```

**配置：**
- 复制 `rust/config.example.json` 为 `rust/config.json`
- 根据环境编辑配置
- 使用 `--config <路径>` 指定自定义配置文件

### Docker 部署

**Node.js：**
```bash
docker build -t aiclient2api:node .
docker run -p 3000:3000 -v $(pwd)/config.json:/app/config.json aiclient2api:node
```

**Rust：**
```bash
cd rust
docker build -t aiclient2api:rust .
docker run -p 3000:3000 -v $(pwd)/config.json:/root/config.json aiclient2api:rust
```

## Kiro 提供商（Claude Sonnet 4）

Kiro 提供商通过 Kiro 客户端免费使用 Claude Sonnet 4 模型。

**关键实现细节：**
- 响应解析使用 4 种回退策略（见 [rust/src/providers/kiro.rs:581-757](rust/src/providers/kiro.rs#L581-L757)）
- 处理 CodeWhisperer 事件流格式
- 自动令牌刷新，带指数退避
- 全面的错误处理和日志记录

**使用 Kiro 运行：**
```bash
# Rust 版本（推荐用于 Kiro）
cd rust
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json

# 使用调试脚本进行故障排除：
./run-kiro-debug.sh
```

**调试 Kiro 问题：**
1. 启用调试日志：`RUST_LOG=debug`
2. 检查日志中的响应解析
3. 详细故障排除见 [rust/KIRO_DEBUG_GUIDE.md](rust/KIRO_DEBUG_GUIDE.md)
4. 使用指南见 [rust/KIRO_USAGE_GUIDE_ZH.md](rust/KIRO_USAGE_GUIDE_ZH.md)

**Kiro 身份验证：**
- 从 https://aibook.ren/archives/kiro-install 下载并安装 Kiro 客户端
- 完成 OAuth 登录生成 `~/.aws/sso/cache/kiro-auth-token.json`
- 在 `config.json` 中配置路径：`kiro_oauth_creds_file_path`

## 测试

### Node.js 测试
- 位置：`src/__tests__/`
- 使用 Jest 框架
- 运行：`npm test`

### Rust 测试
- 单元测试：与源代码内联（`#[cfg(test)]` 模块）
- 集成测试：[rust/tests/](rust/tests/)
- 运行：`cargo test`

**测试文件：**
- [rust/tests/integration_tests.rs](rust/tests/integration_tests.rs) - API 端点测试
- [rust/tests/provider_tests.rs](rust/tests/provider_tests.rs) - 提供商功能测试
- [rust/tests/conversion_tests.rs](rust/tests/conversion_tests.rs) - 格式转换测试
- [rust/tests/system_prompt_tests.rs](rust/tests/system_prompt_tests.rs) - 系统提示词处理测试

## 配置文件

- `config.json` - 主配置文件（参考 `config.example.json`）
- `provider_pools.json` - 多账号池配置
- `input_system_prompt.txt` - 系统提示词注入文件

**关键配置选项：**
- `model_provider` - 使用的 AI 提供商
- `required_api_key` - 身份验证 API 密钥
- `prompt_log_mode` - 日志模式："console"、"file" 或 "none"
- `system_prompt_mode` - "overwrite"（覆盖）或 "append"（追加）
- `provider_pools_file_path` - 账号池配置文件路径

## OAuth 凭据路径

OAuth 凭据的默认位置：
- **Gemini**：`~/.gemini/oauth_creds.json`
- **Kiro**：`~/.aws/sso/cache/kiro-auth-token.json`
- **Qwen**：`~/.qwen/oauth_creds.json`

可在 `config.json` 中覆盖：
- `gemini_oauth_creds_file_path`
- `kiro_oauth_creds_file_path`
- `qwen_oauth_creds_file_path`

## 添加新提供商

### Node.js：
1. 在 `src/` 中创建提供商目录（如 `src/newprovider/`）
2. 实现 `newprovider-core.js`，包含 OAuth 和 API 逻辑
3. 实现 `newprovider-strategy.js`，继承基础策略
4. 在 [src/adapter.js](src/adapter.js) 和 [src/common.js](src/common.js) 中注册
5. 在 `src/__tests__/` 中添加测试

### Rust：
1. 创建 `rust/src/providers/newprovider.rs`
2. 实现 `ApiServiceAdapter` trait
3. 添加 OAuth 处理和 API 调用
4. 在 [rust/src/providers/mod.rs](rust/src/providers/mod.rs) 中注册
5. 更新 [rust/src/adapter.rs](rust/src/adapter.rs) 工厂
6. 在 `rust/tests/provider_tests.rs` 中添加测试

## API 端点

### OpenAI 兼容：
- `POST /v1/chat/completions` - 聊天补全
- `GET /v1/models` - 列出模型

### Claude 兼容：
- `POST /v1/messages` - 消息生成

### Gemini 兼容：
- `GET /v1beta/models` - 列出模型
- `POST /v1beta/models/{model}:generateContent` - 生成内容
- `POST /v1beta/models/{model}:streamGenerateContent` - 流式生成内容

### 工具端点：
- `GET /health` - 健康检查
- `GET /` - 欢迎页面

### 基于路径的提供商路由

可通过 URL 路径指定提供商：
- `http://localhost:3000/claude-custom/v1/messages`
- `http://localhost:3000/claude-kiro-oauth/v1/messages`
- `http://localhost:3000/gemini-cli-oauth/v1/chat/completions`
- `http://localhost:3000/openai-custom/v1/chat/completions`
- `http://localhost:3000/openai-qwen-oauth/v1/chat/completions`

这对 Claude Code 和其他 AI 编程助手很有用。

## 性能考虑

**Rust vs Node.js：**
- 启动速度：Rust 快 4 倍（~50ms vs ~200ms）
- 内存占用：Rust 少 4 倍（~20MB vs ~80MB）
- 请求延迟：Rust 低 40%（60ms vs 100ms）
- 吞吐量：Rust 高 3 倍（15k vs 5k req/s）

**何时使用 Rust：**
- 需要高性能的生产部署
- 资源受限的环境
- 偏好单一二进制部署

**何时使用 Node.js：**
- 快速开发和迭代
- 现有 Node.js 基础设施
- 熟悉 JavaScript 生态系统

## 重要说明

- 两个版本共享相同的 API 契约，完全兼容
- 配置文件格式在两个版本之间相同
- Kiro 提供商在 Rust 版本中效果最佳，因为响应解析经过优化
- Rust 生产构建务必使用 `--release` 标志
- Rust 日志设置 `RUST_LOG=info` 或 `RUST_LOG=debug`
- OAuth 令牌由 cron 任务自动刷新（可配置）
- 账号池通过多个凭据实现高可用性

## 文档

**通用文档：**
- [README.md](README.md) - 项目概述
- [BUILD.md](BUILD.md) - 构建说明（Go 版本，已过时）

**Rust 专用文档：**
- [rust/README.md](rust/README.md) - Rust 版本概述
- [rust/QUICKSTART.md](rust/QUICKSTART.md) - 快速开始指南
- [rust/ARCHITECTURE.md](rust/ARCHITECTURE.md) - 架构详情
- [rust/BUILD_AND_RUN.md](rust/BUILD_AND_RUN.md) - 构建和运行指南
- [rust/KIRO_DEBUG_GUIDE.md](rust/KIRO_DEBUG_GUIDE.md) - Kiro 调试
- [rust/KIRO_USAGE_GUIDE_ZH.md](rust/KIRO_USAGE_GUIDE_ZH.md) - Kiro 使用指南（中文）
- [rust/PERFORMANCE.md](rust/PERFORMANCE.md) - 性能分析

## 故障排除

**Node.js：**
- 使用 `--log-prompts console` 检查日志
- 验证 OAuth 凭据存在且有效
- 测试：`curl http://localhost:3000/health`

**Rust：**
- 启用调试日志：`RUST_LOG=debug cargo run`
- 检查配置文件路径和格式
- Kiro 问题见 [rust/KIRO_DEBUG_GUIDE.md](rust/KIRO_DEBUG_GUIDE.md)
- 使用 `cargo check` 验证编译
- 使用 `cargo clippy` 检查 linting 问题

**常见问题：**
- OAuth 令牌过期：令牌会自动刷新，检查 cron 设置
- API 密钥不匹配：验证配置中的 `required_api_key`
- 端口已被占用：使用 `--port` 或在配置中更改端口
- 找不到提供商：检查配置中 `model_provider` 的拼写
