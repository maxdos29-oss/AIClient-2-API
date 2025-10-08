# Kiro API 调试指南 / Kiro API Debug Guide

## 问题: Cline 报错 "Invalid API Response"

如果你遇到 Cline 错误："Invalid API Response: The provider returned an empty or unparsable response"，请按照以下步骤调试。

### 步骤 1: 启用详细日志

运行程序时设置 `RUST_LOG` 环境变量以查看详细日志：

```bash
cd rust
RUST_LOG=debug cargo run --release -- --config config.json
```

或者如果已经编译好：

```bash
cd rust
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json
```

### 步骤 2: 检查日志输出

查看日志中的以下关键信息：

#### 2.1 响应接收日志
```
[INFO] Raw response length: XXXX bytes
```
这表明我们收到了响应。如果长度为 0，说明 Kiro API 没有返回任何内容。

#### 2.2 响应解析开始
```
[INFO] Starting to parse CodeWhisperer response, length: XXXX
```

#### 2.3 解析策略
- **Strategy 0**: 尝试作为完整 JSON 解析
  - 成功: `[INFO] Successfully parsed response as complete JSON`
  - 失败: 继续下一个策略

- **Strategy 1**: 使用正则表达式解析事件块
  - `[INFO] Found X event blocks via regex`
  - `[INFO] Found content in event: X chars`
  - `[INFO] Found tool call: XXX with id YYY`

- **Strategy 2**: 查找 assistantResponseMessage
  - `[INFO] Found assistantResponseMessage at position X`
  - `[INFO] Extracted content from assistantResponseMessage: X chars`

- **Strategy 3**: 查找任何 JSON 对象中的 content 字段
  - `[INFO] Found content #X: Y chars`

#### 2.4 解析结果
```
[INFO] Successfully parsed: X chars content, Y tool calls
```
或
```
[ERROR] Could not parse any content from CodeWhisperer response!
[ERROR] Full response (first 2000 chars): ...
```

#### 2.5 最终响应
```
[INFO] Returning Claude response with X content blocks
```

### 步骤 3: 常见问题和解决方案

#### 问题 1: 响应长度为 0
**原因**: Kiro API 没有返回任何内容
**解决方案**:
- 检查认证 token 是否有效
- 检查 token 是否过期
- 查看是否有 403 错误

#### 问题 2: 无法解析任何内容
**原因**: CodeWhisperer 响应格式可能已更改
**解决方案**:
1. 查看 `[ERROR] Full response (first 2000 chars)` 日志
2. 将完整响应保存到文件并检查格式
3. 在 GitHub 上报告问题并附上响应样例（隐藏敏感信息）

#### 问题 3: Token 过期
**日志显示**:
```
[INFO] Token expired, attempting refresh...
[INFO] Received 403 Forbidden. Attempting token refresh and retrying...
```
**解决方案**:
- 程序会自动尝试刷新 token
- 如果刷新失败，请手动重新认证

### 步骤 4: 保存原始响应以供分析

修改代码临时保存响应（仅用于调试）：

```bash
# 在 src/providers/kiro.rs 的第 774 行附近，响应接收后添加：
# 注意：这只是示例，不要在生产环境使用
tokio::fs::write("/tmp/kiro_raw_response.txt", &response_text).await.ok();
```

然后检查 `/tmp/kiro_raw_response.txt` 文件的内容。

### 步骤 5: 测试基本连接

创建一个简单的测试请求：

```bash
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "messages": [
      {"role": "user", "content": "Hello"}
    ],
    "max_tokens": 100
  }'
```

### 步骤 6: 验证配置文件

检查 `config.json` 中的 Kiro 配置：

```json
{
  "model_provider": "claude-kiro-oauth",
  "kiro": {
    "oauth_creds_base64": "YOUR_BASE64_ENCODED_CREDENTIALS",
    // 或者
    "oauth_creds_file": "/path/to/kiro-auth-token.json"
  }
}
```

确保 credentials 文件包含：
- `accessToken`
- `refreshToken`
- `expiresAt`
- `profileArn`
- `region`

### 步骤 7: 检查 Kiro 认证文件

认证文件应该类似：

```json
{
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "expiresAt": "2025-10-08T12:00:00.000Z",
  "profileArn": "arn:aws:codewhisperer:us-east-1:...",
  "region": "us-east-1",
  "authMethod": "social",
  "provider": "kiro"
}
```

### 步骤 8: 重新编译和运行

```bash
cd rust
cargo clean
cargo build --release
RUST_LOG=info ./target/release/aiclient2api-rust --config config.json
```

### 获取帮助

如果以上步骤都无法解决问题，请：

1. 收集以下信息：
   - 完整的日志输出（使用 `RUST_LOG=debug`）
   - 配置文件（隐藏敏感信息）
   - 错误发生时的请求内容
   - 原始响应（如果可以保存）

2. 在 GitHub Issues 中报告问题：
   - 描述问题
   - 附上日志和配置
   - 说明你尝试过的解决方案

## 额外提示

### 使用更高的日志级别

```bash
# 只显示 info 及以上级别
RUST_LOG=info ./target/release/aiclient2api-rust

# 显示特定模块的调试日志
RUST_LOG=aiclient2api_rust::providers::kiro=debug ./target/release/aiclient2api-rust

# 显示所有调试日志（可能很多）
RUST_LOG=debug ./target/release/aiclient2api-rust
```

### 日志文件输出

将日志保存到文件：

```bash
RUST_LOG=debug ./target/release/aiclient2api-rust 2>&1 | tee kiro_debug.log
```

### 性能监控

如果响应很慢，检查：
- 网络连接到 AWS CodeWhisperer 服务器
- Token 刷新是否频繁发生
- 是否有多次重试

---

**最后更新**: 2025-10-08

