# Kiro API 使用指南

## 改进说明

最新版本的 Rust 实现针对 Kiro API 响应解析进行了以下改进：

### 1. 增强的日志记录
- 添加了详细的 `info!` 和 `debug!` 级别日志
- 每个解析步骤都有清晰的日志输出
- 解析失败时会输出原始响应（前 2000 个字符）

### 2. 多策略解析
实现了 4 种解析策略（按顺序尝试）：

- **策略 0**: 尝试将整个响应解析为完整的 JSON
- **策略 1**: 使用改进的正则表达式解析 AWS 事件流格式
- **策略 2**: 查找 `assistantResponseMessage` 字段
- **策略 3**: 搜索任何包含 `content` 字段的 JSON 对象

### 3. 容错处理
- 即使所有解析策略都失败，也会返回有效的 Claude 格式响应
- 返回带有提示信息的文本，而不是空响应
- 这样 Cline 就不会显示 "unparsable response" 错误

### 4. 改进的正则表达式
使用了与 JavaScript 版本类似的更灵活的正则表达式模式：
```rust
r"(?s)event(\{.*?(?=event\{|$))"
```
这可以更好地匹配 AWS CodeWhisperer 的事件流格式。

## 运行服务器

### 方法 1: 使用调试模式运行（推荐用于调试）

```bash
cd rust
RUST_LOG=debug cargo run --release -- --config config.json
```

或者如果已经编译：

```bash
cd rust
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json
```

### 方法 2: 使用 info 日志级别运行（日常使用）

```bash
cd rust
RUST_LOG=info ./target/release/aiclient2api-rust --config config.json
```

### 方法 3: 在后台运行并保存日志

```bash
cd rust
nohup RUST_LOG=info ./target/release/aiclient2api-rust --config config.json > kiro.log 2>&1 &
```

查看日志：
```bash
tail -f rust/kiro.log
```

## 配置文件示例

`config.json`:
```json
{
  "host": "127.0.0.1",
  "port": 8080,
  "model_provider": "claude-kiro-oauth",
  "required_api_key": "your-secret-key-here",
  "kiro": {
    "oauth_creds_base64": "YOUR_BASE64_ENCODED_CREDENTIALS"
  }
}
```

或者使用文件路径：
```json
{
  "host": "127.0.0.1",
  "port": 8080,
  "model_provider": "claude-kiro-oauth",
  "required_api_key": "your-secret-key-here",
  "kiro": {
    "oauth_creds_file": "/Users/yourname/.aws/sso/cache/kiro-auth-token.json"
  }
}
```

## 在 Cline 中测试

1. **启动服务器**（在终端中）:
   ```bash
   cd rust
   RUST_LOG=info ./target/release/aiclient2api-rust --config config.json
   ```

2. **配置 Cline**:
   - 打开 Cline 设置
   - API Provider: 选择 "Claude"
   - API Key: 输入你在 `config.json` 中设置的 `required_api_key`
   - Base URL: `http://localhost:8080`
   - Model: 选择 `claude-sonnet-4-20250514` 或其他支持的模型

3. **发送测试消息**:
   - 在 Cline 中输入简单的问题，例如 "Hello, how are you?"
   - 观察终端中的日志输出

4. **查看日志输出**:
   你应该看到类似以下的日志：
   ```
   [INFO] Received Claude messages request
   [INFO] Kiro generate_content
   [INFO] Calling Kiro API: https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse
   [INFO] Raw response length: 1234 bytes
   [INFO] Starting to parse CodeWhisperer response, length: 1234
   [INFO] Found X event blocks via regex
   [INFO] Found content in event: Y chars
   [INFO] Successfully parsed: Y chars content, 0 tool calls
   [INFO] Returning Claude response with 1 content blocks
   [INFO] Claude messages request completed successfully
   ```

## 如果仍然遇到问题

### 检查 1: 验证服务器正在运行
```bash
curl http://localhost:8080/health
```
应该返回：
```json
{
  "status": "healthy",
  "timestamp": "2025-10-08T...",
  "provider": "claude-kiro-oauth"
}
```

### 检查 2: 测试基本 API 调用
```bash
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-secret-key-here" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "messages": [
      {"role": "user", "content": "Say hello"}
    ],
    "max_tokens": 100
  }'
```

### 检查 3: 查看详细日志
如果响应有问题，查找以下关键日志：

1. **响应接收**:
   ```
   [INFO] Raw response length: X bytes
   ```
   如果 X = 0，说明没有收到响应

2. **解析尝试**:
   ```
   [INFO] Starting to parse CodeWhisperer response
   ```

3. **解析成功**:
   ```
   [INFO] Successfully parsed: X chars content, Y tool calls
   ```

4. **解析失败**:
   ```
   [ERROR] Could not parse any content from CodeWhisperer response!
   [ERROR] Full response (first 2000 chars): ...
   ```
   将这个响应内容保存下来，并报告问题

### 检查 4: Token 过期
如果看到：
```
[INFO] Token expired, attempting refresh...
```
或
```
[INFO] Received 403 Forbidden. Attempting token refresh and retrying...
```

这是正常的，程序会自动刷新 token。如果刷新失败，你需要重新生成 Kiro 认证 token。

## 支持的模型

- `claude-sonnet-4-20250514`
- `claude-sonnet-4-5-20250929`
- `claude-3-7-sonnet-20250219`
- `claude-3-5-sonnet-20241022`
- `claude-3-5-haiku-20241022`
- `amazonq-claude-sonnet-4-20250514`
- `amazonq-claude-sonnet-4-5-20250929`
- `amazonq-claude-3-7-sonnet-20250219`

## 性能提示

1. **编译优化**: 使用 `--release` 标志编译以获得最佳性能
2. **日志级别**: 生产环境使用 `RUST_LOG=info` 而不是 `debug`
3. **超时设置**: 默认超时为 300 秒（5 分钟），足够大多数请求

## 故障排除

详细的调试步骤请参考：[KIRO_DEBUG_GUIDE.md](./KIRO_DEBUG_GUIDE.md)

## 报告问题

如果遇到无法解决的问题，请收集以下信息并在 GitHub Issues 中报告：

1. 完整的日志输出（使用 `RUST_LOG=debug`）
2. 配置文件（隐藏敏感信息）
3. 错误发生时的请求内容
4. Cline 显示的错误消息
5. 服务器版本信息

```bash
./target/release/aiclient2api-rust --version
```

---

**提示**: 如果响应中看到 "⚠️ Unable to parse response from Kiro API..."，这意味着解析失败。请查看服务器日志中的 `[ERROR]` 行以获取更多详细信息。

