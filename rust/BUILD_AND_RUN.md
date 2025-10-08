# 构建和运行指南

## 快速开始

### 1. 安装 Rust

如果还没有安装 Rust，请访问 https://rustup.rs/ 并按照说明安装。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 克隆项目

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
```

### 3. 配置

复制示例配置文件：

```bash
cp config.example.json config.json
cp provider_pools.example.json provider_pools.json
```

编辑 `config.json` 填入你的 API 密钥和配置。

### 4. 构建项目

```bash
# 开发模式构建
cargo build

# 发布模式构建（优化）
cargo build --release
```

### 5. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_conversion

# 显示测试输出
cargo test -- --nocapture

# 运行测试并显示详细信息
cargo test -- --test-threads=1 --nocapture
```

### 6. 运行服务器

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release

# 或者直接运行编译后的二进制
./target/release/aiclient2api-rust
```

### 7. 测试 API

#### OpenAI 格式

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer 123456" \
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

#### Claude 格式

```bash
curl -X POST http://localhost:3000/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: 123456" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-3-opus",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 1024
  }'
```

#### Gemini 格式

```bash
curl -X POST "http://localhost:3000/v1beta/models/gemini-2.5-flash:generateContent?key=123456" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{
      "role": "user",
      "parts": [{"text": "Hello!"}]
    }]
  }'
```

#### 健康检查

```bash
curl http://localhost:3000/health
```

## 开发工作流

### 代码检查

```bash
# 快速检查（不生成二进制）
cargo check

# 类型检查和基本错误
cargo clippy

# 代码格式化
cargo fmt

# 检查格式是否正确（不修改）
cargo fmt -- --check
```

### 监控文件变化自动重新编译

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 监控并自动运行
cargo watch -x run

# 监控并自动测试
cargo watch -x test
```

### 性能分析

```bash
# 使用 flamegraph 进行性能分析
cargo install flamegraph

# 生成火焰图
cargo flamegraph
```

### 代码覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html
```

## Docker 部署

### 构建 Docker 镜像

```bash
docker build -t aiclient2api-rust .
```

### 运行 Docker 容器

```bash
docker run -d \
  --name aiclient2api \
  -p 3000:3000 \
  -v $(pwd)/config.json:/app/config.json:ro \
  -v $(pwd)/provider_pools.json:/app/provider_pools.json:ro \
  aiclient2api-rust
```

### 使用 Docker Compose

```bash
# 启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止
docker-compose down
```

## 环境变量

```bash
# 设置日志级别
export RUST_LOG=debug

# 设置 HTTP 代理
export HTTP_PROXY=http://proxy:port
export HTTPS_PROXY=http://proxy:port

# 运行
cargo run
```

## 故障排查

### 编译错误

如果遇到编译错误：

1. 确保 Rust 版本 >= 1.70
   ```bash
   rustc --version
   rustup update
   ```

2. 清理并重新构建
   ```bash
   cargo clean
   cargo build
   ```

### 运行时错误

1. 检查配置文件格式是否正确
2. 确保所需的 API 密钥都已配置
3. 查看详细日志：`RUST_LOG=debug cargo run`

### 测试失败

```bash
# 运行失败的测试并显示详细输出
cargo test failing_test_name -- --nocapture

# 更新测试快照（如果使用）
cargo test -- --ignored
```

## 性能优化

### 发布构建优化

已在 `Cargo.toml` 中配置：

```toml
[profile.release]
opt-level = 3          # 最高优化
lto = true             # 链接时优化
codegen-units = 1      # 单代码生成单元
strip = true           # 移除调试符号
```

### 运行时优化建议

1. 使用发布模式：`cargo run --release`
2. 配置足够的 Tokio 工作线程
3. 启用 HTTP/2
4. 使用连接池

## 常见问题

### Q: 如何更改端口？

A: 编辑 `config.json` 中的 `port` 字段。

### Q: 如何添加新的 AI 提供商？

A: 
1. 在 `src/providers/` 创建新文件
2. 实现 `ApiServiceAdapter` trait
3. 在 `adapter.rs` 添加工厂方法
4. 更新 `ModelProvider` 枚举

### Q: 如何启用详细日志？

A: 设置环境变量 `RUST_LOG=debug` 或 `RUST_LOG=trace`

### Q: 性能如何？

A: Rust 版本通常比 Node.js 版本快 2-5 倍，内存占用减少 50-70%。

## 下一步

- 阅读 [ARCHITECTURE.md](./ARCHITECTURE.md) 了解架构设计
- 查看 [CONTRIBUTING.md](./CONTRIBUTING.md) 参与贡献
- 阅读 [README.md](./README.md) 了解更多功能

---

祝你使用愉快！🦀

