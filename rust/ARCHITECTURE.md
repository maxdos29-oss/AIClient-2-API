# AIClient-2-API Rust 架构说明

## 项目概述

AIClient-2-API Rust 版本是原 Node.js 项目的完整重写，采用 Rust 语言实现，保持 API 兼容性的同时提供更高的性能和更强的类型安全保证。

## 技术栈

### 核心框架
- **Axum**: 现代化的 Rust Web 框架，基于 Tokio 异步运行时
- **Tokio**: 异步运行时，提供高性能的异步 I/O
- **Tower**: 中间件和服务抽象层
- **Hyper**: 底层 HTTP 实现

### 序列化和数据处理
- **Serde**: JSON 序列化/反序列化
- **serde_json**: JSON 处理

### HTTP 客户端
- **Reqwest**: 异步 HTTP 客户端，用于调用外部 AI API

### 认证
- **OAuth2**: OAuth 2.0 客户端实现
- **yup-oauth2**: Google OAuth 专用库

### 其他工具
- **UUID**: 生成唯一标识符
- **Chrono**: 日期时间处理
- **Tracing**: 结构化日志
- **Anyhow/Thiserror**: 错误处理

## 项目结构

```
rust/
├── Cargo.toml                 # 项目依赖配置
├── src/
│   ├── main.rs               # 程序入口
│   ├── config.rs             # 配置管理
│   ├── server.rs             # HTTP 服务器和路由
│   ├── common.rs             # 通用类型和工具函数
│   ├── adapter.rs            # API 适配器接口定义
│   ├── convert.rs            # 格式转换逻辑
│   ├── pool_manager.rs       # 账号池管理器
│   ├── strategies.rs         # 提供商策略模式
│   └── providers/            # 各提供商实现
│       ├── mod.rs
│       ├── gemini.rs         # Google Gemini
│       ├── openai.rs         # OpenAI
│       ├── claude.rs         # Anthropic Claude
│       ├── kiro.rs           # Kiro API
│       └── qwen.rs           # Qwen Code
├── config.example.json        # 配置文件示例
├── provider_pools.example.json # 账号池配置示例
├── Dockerfile                 # Docker 镜像构建
├── docker-compose.yml         # Docker Compose 配置
└── README.md                  # 项目文档
```

## 核心模块设计

### 1. 配置管理 (config.rs)

负责加载和管理服务器配置：

```rust
pub struct Config {
    // 服务器配置
    pub host: String,
    pub port: u16,
    pub required_api_key: String,
    
    // 模型提供商配置
    pub model_provider: String,
    pub default_model_providers: Vec<String>,
    
    // 各提供商的凭据
    pub openai_api_key: Option<String>,
    pub claude_api_key: Option<String>,
    // ...
    
    // 系统提示词配置
    pub system_prompt_file_path: PathBuf,
    pub system_prompt_mode: String,
    
    // 账号池配置
    pub provider_pools: HashMap<String, Vec<ProviderConfig>>,
}
```

特点：
- 支持从 JSON 文件加载配置
- 支持环境变量覆盖
- 自动验证和规范化配置

### 2. HTTP 服务器 (server.rs)

实现 HTTP 服务器和路由处理：

```rust
pub async fn start_server(config: Config) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/chat/completions", post(openai_chat_handler))
        .route("/v1/models", get(openai_models_handler))
        .route("/v1/messages", post(claude_messages_handler))
        .route("/v1beta/models", get(gemini_models_handler))
        .route("/v1beta/models/:model/:action", post(gemini_content_handler))
        .with_state(Arc::new(AppState { config }))
        .layer(cors);
    
    // ... 启动服务器
}
```

特点：
- 使用 Axum 框架
- 支持 CORS
- 自动错误处理
- 路径参数和查询参数解析

### 3. 通用类型 (common.rs)

定义核心数据结构：

```rust
// 模型协议
pub enum ModelProtocol {
    Gemini,
    OpenAI,
    Claude,
}

// 模型提供商
pub enum ModelProvider {
    GeminiCliOAuth,
    OpenAICustom,
    ClaudeCustom,
    ClaudeKiroOAuth,
    OpenAIQwenOAuth,
}

// API 请求
pub struct ApiRequest {
    pub model: Option<String>,
    pub messages: Option<Vec<Message>>,
    pub contents: Option<Vec<Content>>,
    // ...
}

// API 响应
pub struct ApiResponse {
    pub data: HashMap<String, Value>,
}
```

特点：
- 强类型定义
- Serde 序列化支持
- 支持多种 API 格式

### 4. 适配器模式 (adapter.rs)

定义统一的服务适配器接口：

```rust
#[async_trait]
pub trait ApiServiceAdapter: Send + Sync {
    async fn generate_content(&self, model: &str, request_body: Value) -> Result<Value>;
    async fn generate_content_stream(&self, model: &str, request_body: Value) 
        -> Result<Pin<Box<dyn Stream<Item = Result<Value>> + Send>>>;
    async fn list_models(&self) -> Result<ModelListResponse>;
    async fn refresh_token(&self) -> Result<()>;
}
```

特点：
- 抽象统一接口
- 支持流式和非流式响应
- 异步操作
- 可扩展性强

### 5. 格式转换 (convert.rs)

处理不同 API 格式之间的转换：

```rust
pub fn convert_data(
    data: Value,
    conversion_type: ConversionType,
    from_protocol: ModelProtocol,
    to_protocol: ModelProtocol,
    model: Option<&str>,
) -> Result<Value> {
    // 转换逻辑
}
```

支持的转换：
- OpenAI ↔ Gemini
- OpenAI ↔ Claude
- Claude ↔ Gemini

转换类型：
- Request（请求）
- Response（响应）
- StreamChunk（流式数据块）
- ModelList（模型列表）

### 6. 账号池管理 (pool_manager.rs)

实现多账号管理和负载均衡：

```rust
pub struct ProviderPoolManager {
    pools: Arc<RwLock<HashMap<String, Vec<ProviderStatus>>>>,
    round_robin_index: Arc<RwLock<HashMap<String, usize>>>,
    max_error_count: u32,
}

impl ProviderPoolManager {
    pub async fn select_provider(&self, provider_type: &str) -> Option<ProviderConfig>;
    pub async fn mark_provider_unhealthy(&self, provider_type: &str, uuid: &str);
    pub async fn mark_provider_healthy(&self, provider_type: &str, uuid: &str);
    pub async fn perform_health_checks(&self);
}
```

特点：
- 轮询负载均衡
- 健康检查
- 故障转移
- 并发安全（使用 RwLock）

### 7. 策略模式 (strategies.rs)

为不同提供商实现特定的处理策略：

```rust
#[async_trait]
pub trait ProviderStrategy: Send + Sync {
    fn extract_model_and_stream_info(&self, request: &Value) -> Result<(String, bool)>;
    fn extract_response_text(&self, response: &Value) -> Result<String>;
    fn extract_prompt_text(&self, request: &Value) -> Result<String>;
    async fn apply_system_prompt_from_file(
        &self,
        request: Value,
        system_prompt: Option<&str>,
        mode: &str,
    ) -> Result<Value>;
}
```

实现类：
- `GeminiStrategy`
- `OpenAIStrategy`
- `ClaudeStrategy`

### 8. 提供商实现 (providers/)

每个 AI 服务提供商的具体实现：

```rust
pub struct GeminiApiService {
    // 提供商特定的字段
}

#[async_trait]
impl ApiServiceAdapter for GeminiApiService {
    // 实现接口方法
}
```

提供商：
- Gemini (Google)
- OpenAI
- Claude (Anthropic)
- Kiro
- Qwen

## 数据流

### 请求处理流程

```
Client Request
    ↓
HTTP Server (Axum)
    ↓
Authorization Check
    ↓
Route Handler
    ↓
Provider Path Override (optional)
    ↓
Pool Manager (select provider)
    ↓
Format Conversion (if needed)
    ↓
Provider Adapter
    ↓
External API Call
    ↓
Response Conversion (if needed)
    ↓
Client Response
```

### 流式响应处理

```
Client Request
    ↓
Server Handler
    ↓
Provider Adapter (generate_content_stream)
    ↓
AsyncStream
    ↓
Format Conversion (per chunk)
    ↓
SSE Stream to Client
```

## 并发模型

### Tokio 异步运行时

- 使用 `#[tokio::main]` 启动异步运行时
- 所有 I/O 操作都是异步的
- 使用 `async/await` 语法

### 并发安全

- 使用 `Arc<T>` 共享状态
- 使用 `RwLock<T>` 或 `Mutex<T>` 保护可变状态
- 无数据竞争（Rust 编译器保证）

## 错误处理

### 错误类型

```rust
pub enum AppError {
    Unauthorized,
    BadRequest(String),
    InternalError(anyhow::Error),
}
```

### 错误转换

- 使用 `anyhow::Error` 作为通用错误类型
- 实现 `From<E>` trait 自动转换
- 使用 `?` 操作符简化错误传播

### 错误响应

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::InternalError(e) => {
                error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        (status, Json(json!({ "error": { "message": message } }))).into_response()
    }
}
```

## 性能优化

### 编译优化

```toml
[profile.release]
opt-level = 3          # 最高优化级别
lto = true             # 链接时优化
codegen-units = 1      # 单个代码生成单元
strip = true           # 剥离调试符号
```

### 运行时优化

- 使用 Zero-Copy 技术（避免不必要的数据复制）
- 使用 `Arc` 而非克隆大型数据结构
- 异步 I/O（避免阻塞）
- 连接池（HTTP 客户端）

### 内存优化

- 栈分配优先（避免堆分配）
- 使用 `Cow<str>` 延迟克隆
- 及时释放大型数据结构

## 安全性

### 类型安全

- 编译时类型检查
- 无空指针（Option<T>）
- 无数据竞争

### 内存安全

- 所有权系统
- 借用检查器
- 生命周期管理

### API 安全

- API 密钥验证
- CORS 配置
- 敏感信息不记录日志

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        // 测试代码
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_api_endpoint() {
    // 测试 HTTP 端点
}
```

### 性能测试

- Criterion 基准测试
- 压力测试

## 部署

### 本地部署

```bash
cargo build --release
./target/release/aiclient2api-rust
```

### Docker 部署

```bash
docker build -t aiclient2api-rust .
docker run -p 3000:3000 aiclient2api-rust
```

### Docker Compose

```bash
docker-compose up -d
```

## 监控和日志

### 日志框架

使用 `tracing` 框架：

```rust
info!("Server started on {}:{}", host, port);
debug!("Request: {:?}", request);
error!("Error occurred: {}", error);
```

### 日志级别

- `trace`: 最详细的日志
- `debug`: 调试信息
- `info`: 一般信息
- `warn`: 警告
- `error`: 错误

### 环境变量配置

```bash
export RUST_LOG=info,aiclient2api_rust=debug
```

## 扩展性

### 添加新的提供商

1. 在 `providers/` 下创建新文件
2. 实现 `ApiServiceAdapter` trait
3. 在 `adapter.rs` 的 `create_adapter` 函数中添加分支
4. 更新 `ModelProvider` 枚举

### 添加新的转换格式

1. 在 `convert.rs` 中添加转换函数
2. 在 `convert_data` 函数中添加匹配分支
3. 实现双向转换（如果需要）

### 添加新的策略

1. 在 `strategies.rs` 中创建新的策略结构体
2. 实现 `ProviderStrategy` trait
3. 在工厂函数中添加创建逻辑

## 与 Node.js 版本的对比

| 特性 | Node.js 版本 | Rust 版本 |
|------|-------------|----------|
| 性能 | 中等 | 高 |
| 内存占用 | 较高 | 低 |
| 启动速度 | 快 | 非常快 |
| 类型安全 | 弱（TypeScript可选） | 强（编译时） |
| 错误处理 | 异常 | Result<T, E> |
| 并发模型 | 事件循环 | Tokio异步 |
| 依赖管理 | npm | Cargo |
| 部署 | 需要Node.js运行时 | 单一二进制 |

## 未来计划

- [ ] 完整实现所有提供商的核心功能
- [ ] 实现完整的格式转换逻辑
- [ ] 添加更多的单元测试和集成测试
- [ ] 实现 WebSocket 支持
- [ ] 添加监控和指标收集
- [ ] 性能基准测试和优化
- [ ] 支持插件系统
- [ ] GraphQL API 支持

## 参考资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 文档](https://tokio.rs/)
- [Axum 文档](https://docs.rs/axum/)
- [Serde 文档](https://serde.rs/)
- [OpenAI API 文档](https://platform.openai.com/docs/api-reference)
- [Anthropic API 文档](https://docs.anthropic.com/)
- [Google Gemini API 文档](https://ai.google.dev/docs)

