# Kiro API 响应解析问题修复总结

## 问题描述

Cline 报错：`Invalid API Response: The provider returned an empty or unparsable response.`

这个错误表明 Kiro API（CodeWhisperer）返回的响应无法被正确解析。

## 根本原因

1. **解析策略不够灵活**: 原有的正则表达式无法正确匹配所有 CodeWhisperer 事件流格式
2. **日志不够详细**: 无法诊断响应解析失败的具体原因
3. **空响应处理不当**: 当解析失败时返回空内容，导致 Cline 报错

## 实施的改进

### 1. 增强日志记录 (`rust/src/providers/kiro.rs`)

添加了详细的日志输出，便于诊断问题：

```rust
info!("Starting to parse CodeWhisperer response, length: {}", response_text.len());
debug!("First 1000 chars: {}", &response_text[..response_text.len().min(1000)]);
```

在每个解析步骤都添加了日志：
- 响应接收: `[INFO] Raw response length: X bytes`
- 解析开始: `[INFO] Starting to parse CodeWhisperer response`
- 策略尝试: `[INFO] Found X event blocks via regex`
- 解析成功: `[INFO] Successfully parsed: X chars content, Y tool calls`
- 解析失败: `[ERROR] Could not parse any content from CodeWhisperer response!`

### 2. 多策略解析

实现了 4 种解析策略（按顺序尝试）：

#### 策略 0: 完整 JSON 解析
```rust
if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(response_text) {
    // 尝试提取 assistantResponseMessage 或 content
}
```

#### 策略 1: 改进的正则表达式
使用更灵活的模式，类似 JavaScript 实现：
```rust
regex::Regex::new(r"(?s)event(\{.*?(?=event\{|$))")
```
- `(?s)` 标志使 `.` 匹配换行符
- `.*?` 非贪婪匹配
- `(?=event\{|$)` 前瞻断言，匹配到下一个 event{ 或字符串结尾

#### 策略 2: 查找 assistantResponseMessage
```rust
if let Some(pos) = response_text.find("\"assistantResponseMessage\"") {
    // 提取 content 字段
}
```

#### 策略 3: 搜索任何 content 字段
```rust
while let Some(pos) = response_text[start..].find("\"content\":\"") {
    // 提取所有 content 值
}
```

### 3. 容错处理

即使所有解析策略都失败，也返回有效响应：

```rust
if content_array.is_empty() {
    error!("No content parsed from CodeWhisperer response!");
    content_array.push(json!({
        "type": "text",
        "text": "⚠️ Unable to parse response from Kiro API. The response may be in an unexpected format. Please check the server logs with RUST_LOG=debug for details."
    }));
}
```

这样 Cline 会收到有效的响应，显示错误提示而不是报"unparsable response"错误。

### 4. 改进的事件块解析

```rust
let mut search_pos = 0;
while search_pos < bytes.len() {
    if let Some(relative_pos) = block_text[search_pos..].find('}') {
        let json_candidate = &block_text[..brace_pos + 1];
        
        if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(json_candidate) {
            // 处理工具调用或内容事件
            break;
        }
        search_pos = brace_pos + 1;
    } else {
        break;
    }
}
```

## 文档和工具

### 创建的文档

1. **KIRO_DEBUG_GUIDE.md** - 详细的调试指南
   - 如何启用详细日志
   - 日志输出含义解释
   - 常见问题和解决方案
   - 故障排除步骤

2. **KIRO_USAGE_GUIDE_ZH.md** - 中文使用指南
   - 改进说明
   - 运行服务器的方法
   - Cline 配置步骤
   - 测试和验证方法

3. **run-kiro-debug.sh** - 调试启动脚本
   - 自动检查配置
   - 显示配置信息
   - 启用调试日志
   - 日志输出到文件和终端

## 使用方法

### 快速启动（推荐）

```bash
cd rust
./run-kiro-debug.sh
```

### 手动启动

```bash
cd rust

# 调试模式（详细日志）
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json

# 或保存日志到文件
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json 2>&1 | tee kiro_debug.log
```

### 在 Cline 中配置

1. API Provider: Claude
2. Base URL: `http://localhost:8080`
3. API Key: 你在 config.json 中设置的 `required_api_key`
4. Model: `claude-sonnet-4-20250514` 或其他支持的模型

## 日志示例

### 成功的响应解析

```
[INFO] Received Claude messages request
[INFO] Kiro generate_content
[INFO] Calling Kiro API: https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse
[INFO] Raw response length: 2453 bytes
[INFO] Starting to parse CodeWhisperer response, length: 2453
[INFO] Using regex to parse event blocks
[INFO] Found content in event: 145 chars
[INFO] Found 8 event blocks via regex
[INFO] Successfully parsed: 145 chars content, 0 tool calls
[INFO] Returning Claude response with 1 content blocks
[INFO] Claude messages request completed successfully
```

### 解析失败但有容错处理

```
[ERROR] Could not parse any content from CodeWhisperer response!
[ERROR] Full response (first 2000 chars): event{...}...
[INFO] Returning empty content to prevent Cline parsing error
[INFO] Returning Claude response with 1 content blocks
```

Cline 会收到：
```
⚠️ Unable to parse response from Kiro API. The response may be in an unexpected format. Please check the server logs with RUST_LOG=debug for details.
```

## 测试步骤

### 1. 健康检查

```bash
curl http://localhost:8080/health
```

预期输出：
```json
{
  "status": "healthy",
  "timestamp": "2025-10-08T...",
  "provider": "claude-kiro-oauth"
}
```

### 2. 基本 API 测试

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

### 3. Cline 测试

在 Cline 中输入简单问题，观察：
- 终端日志输出
- Cline 响应是否正常
- 是否还有 "unparsable response" 错误

## 下一步

如果问题仍然存在：

1. **收集日志**: 
   ```bash
   RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json 2>&1 | tee detailed.log
   ```

2. **查看详细响应**:
   - 查找 `[ERROR] Full response` 行
   - 检查响应格式是否符合预期

3. **报告问题**:
   - 提供完整日志
   - 提供配置文件（隐藏敏感信息）
   - 提供错误时的请求内容
   - 提供 CodeWhisperer 原始响应样例

## 技术细节

### 关键代码位置

- **响应解析**: `rust/src/providers/kiro.rs:581-757`
- **API 调用**: `rust/src/providers/kiro.rs:759-872`
- **非流式响应**: `rust/src/providers/kiro.rs:877-884`
- **流式响应**: `rust/src/providers/kiro.rs:886-1035`

### 支持的模型

所有 Claude Sonnet 4 和 3.7 模型：
- claude-sonnet-4-20250514
- claude-sonnet-4-5-20250929
- claude-3-7-sonnet-20250219
- claude-3-5-sonnet-20241022
- claude-3-5-haiku-20241022
- amazonq-* variants

### 性能考虑

- 使用 `--release` 编译优化
- 默认超时: 300秒
- 自动 token 刷新
- 指数退避重试（429、5xx 错误）

## 参考资料

- [KIRO_DEBUG_GUIDE.md](rust/KIRO_DEBUG_GUIDE.md) - 调试指南
- [KIRO_USAGE_GUIDE_ZH.md](rust/KIRO_USAGE_GUIDE_ZH.md) - 使用指南
- [run-kiro-debug.sh](rust/run-kiro-debug.sh) - 启动脚本

## 版本信息

- 修复日期: 2025-10-08
- Rust 版本: 1.40+
- 主要改进: 响应解析、日志记录、错误处理

---

**如果这些改进解决了你的问题，请更新此文档并分享你的经验！**
