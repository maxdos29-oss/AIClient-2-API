# Cline "Invalid API Response" 错误快速修复指南

## 症状

服务器日志显示解析成功：
```
[INFO] Successfully parsed: 506 chars content, 0 tool calls
[INFO] Returning Claude response with 1 content blocks
```

但 Cline 仍然报错：
```
Invalid API Response: The provider returned an empty or unparsable response.
```

## 可能的原因

### 1. 内容被重复提取（最可能）

你的日志显示：
```
Found content #1: 42 chars
Found content #2: 36 chars
...
Found content #22: 13 chars
Found 22 content blocks using JSON object search, total: 506 chars
```

这说明从响应中提取了 22 个 content 字段，可能包括：
- 历史消息中的 content
- 系统提示中的 content
- 最终响应中的 content

**解决方案**: 最新代码已修复，只提取最后一个 `assistantResponseMessage` 的内容。

### 2. 内容格式问题

提取的内容可能包含：
- 控制字符
- JSON 格式错误
- 特殊字符未正确转义

### 3. Cline 配置问题

Cline 可能配置为使用不同的 API 格式。

## 立即测试

### 步骤 1: 重新编译并运行

```bash
cd rust
cargo build --release
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json
```

### 步骤 2: 运行测试脚本

在**另一个终端**中：

```bash
cd rust
./test_kiro_response.sh your-api-key 8080
```

这将：
- 测试健康检查
- 发送简单消息
- 验证响应格式
- 显示完整的响应结构

### 步骤 3: 查看新日志

现在你应该看到：
```
[INFO] Trying to find assistantResponseMessage content field
[INFO] Found last assistantResponseMessage at position XXXX
[INFO] Extracted content from last assistantResponseMessage: XX chars
```

而不是：
```
Found content #1: 42 chars
Found content #2: 36 chars
...
```

### 步骤 4: 检查响应内容

在日志中查找：
```
[DEBUG] Content block 0: type=text, length=XX, preview=...
```

确认内容预览看起来正确。

## 详细诊断

### 查看完整响应结构

在日志中查找：
```
[DEBUG] Response structure: {
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "..."
    }
  ],
  ...
}
```

验证：
1. `type` 是 `"message"`
2. `role` 是 `"assistant"`
3. `content` 是数组
4. `content[0].type` 是 `"text"`
5. `content[0].text` 包含实际内容

### 测试 curl 请求

```bash
curl -X POST http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "messages": [
      {"role": "user", "content": "Say hello"}
    ],
    "max_tokens": 50
  }' | jq .
```

确认输出是有效的 JSON 并符合 Claude API 格式。

### 验证 Cline 配置

在 Cline 设置中确认：

1. **API Provider**: 必须选择 `Claude`（不是 OpenAI）
2. **Model**: 选择 `claude-sonnet-4-20250514` 或其他支持的模型
3. **Base URL**: `http://localhost:8080`（不要加 `/v1/messages`）
4. **API Key**: 与 `config.json` 中的 `required_api_key` 一致

## 常见错误配置

### 错误 1: Base URL 设置错误

❌ 错误:
- `http://localhost:8080/v1/messages`
- `http://localhost:8080/v1/chat/completions`

✅ 正确:
- `http://localhost:8080`

### 错误 2: API Provider 选择错误

❌ 错误:
- 选择 "OpenAI"
- 选择 "Anthropic (Official)"

✅ 正确:
- 选择 "Claude"（自定义端点）

### 错误 3: API Key 不匹配

❌ 错误:
- Cline 中的 API Key 与 `config.json` 中的 `required_api_key` 不一致

✅ 正确:
- 两者必须完全相同

## 高级调试

### 保存响应到文件

临时修改代码以保存原始响应（仅用于调试）：

在 `rust/src/providers/kiro.rs` 的第 806 行附近添加：

```rust
// 临时调试代码 - 保存原始响应
let debug_file = format!("/tmp/kiro_response_{}.json", chrono::Utc::now().timestamp());
tokio::fs::write(&debug_file, serde_json::to_string_pretty(&result)?).await.ok();
info!("Saved response to {}", debug_file);
```

然后检查 `/tmp/kiro_response_*.json` 文件。

### 启用 Axum 日志

```bash
RUST_LOG=debug,axum=trace ./target/release/aiclient2api-rust --config config.json
```

这将显示 HTTP 请求和响应的详细信息。

### 使用 Wireshark/tcpdump

捕获 Cline 和服务器之间的 HTTP 流量：

```bash
sudo tcpdump -i lo0 -A 'port 8080' > traffic.log
```

然后在 Cline 中发送请求，检查 `traffic.log` 文件。

## 如果问题仍然存在

### 收集信息

1. **服务器日志**（完整的 debug 日志）:
   ```bash
   RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json 2>&1 | tee full_debug.log
   ```

2. **测试脚本输出**:
   ```bash
   ./test_kiro_response.sh your-api-key 8080 > test_output.txt 2>&1
   ```

3. **Cline 配置截图**

4. **Cline 完整错误消息**

### 报告问题

在 GitHub Issues 中提供：
- 完整的服务器日志（隐藏敏感信息）
- 测试脚本输出
- Cline 配置和错误消息
- 操作系统和 Cline 版本

## 快速检查清单

- [ ] 重新编译最新代码
- [ ] 配置文件正确（`config.json`）
- [ ] 服务器正常启动（`curl http://localhost:8080/health`）
- [ ] 测试脚本通过（`./test_kiro_response.sh`）
- [ ] Cline API Provider 设置为 "Claude"
- [ ] Cline Base URL 是 `http://localhost:8080`（不含路径）
- [ ] Cline API Key 与配置文件一致
- [ ] 服务器日志显示正确提取内容（只有一个 content block）
- [ ] curl 测试返回有效的 JSON

## 临时解决方案

如果问题无法立即解决，可以暂时使用 Node.js 版本：

```bash
cd ..  # 回到项目根目录
node src/api-server.js
```

Node.js 版本的 Kiro 实现已经过充分测试。

---

**更新日期**: 2025-10-08
**最新改进**: 修复了 Strategy 3，只提取最后一个 assistantResponseMessage 的内容，避免重复提取历史消息

