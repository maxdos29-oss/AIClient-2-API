# 快速修复步骤 - Cline "Invalid API Response" 错误

## 问题
即使服务器日志显示成功解析了响应，Cline 仍然报错：
```
Invalid API Response: The provider returned an empty or unparsable response.
```

## 已实施的修复

我已经修复了代码中的解析问题（Strategy 3 重复提取了历史消息中的内容）。

## 立即执行的步骤

### 1. 停止当前服务器

如果服务器正在运行，按 `Ctrl+C` 停止它。

### 2. 重新编译

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
cargo build --release
```

等待编译完成（可能需要几分钟）。

### 3. 启动调试模式

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json
```

### 4. 在另一个终端测试

打开新终端，运行：

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
./test_kiro_response.sh your-api-key-here 8080
```

将 `your-api-key-here` 替换为你在 `config.json` 中设置的 `required_api_key`。

### 5. 检查测试结果

如果测试通过，你应该看到：
```
✓ 所有测试通过！响应格式正确
```

### 6. 观察新的日志输出

在服务器日志中，你现在应该看到：
```
[INFO] Trying to find assistantResponseMessage content field
[INFO] Found last assistantResponseMessage at position XXXX
[INFO] Extracted content from last assistantResponseMessage: XX chars
[INFO] Successfully parsed: XX chars content, 0 tool calls
```

**关键区别**：
- ❌ 之前: `Found content #1`, `Found content #2`, ... `Found content #22` (提取了22个内容块)
- ✅ 现在: 只提取最后一个 assistantResponseMessage 的内容

### 7. 在 Cline 中测试

确保 Cline 配置正确：

**重要配置检查：**

1. **API Provider**: 
   - ✅ 选择 `Claude` 或 `Anthropic`
   - ❌ 不要选 `OpenAI`

2. **Base URL**: 
   - ✅ `http://localhost:8080`
   - ❌ 不要加 `/v1/messages` 或其他路径

3. **API Key**: 
   - 必须与 `config.json` 中的 `required_api_key` 完全一致

4. **Model**: 
   - 推荐: `claude-sonnet-4-20250514`
   - 或其他支持的模型

然后在 Cline 中发送一个简单的测试消息，例如 "Hello"。

### 8. 查看调试日志

在服务器终端中观察：
```
[INFO] Received Claude messages request
[DEBUG] Content block 0: type=text, length=XX, preview=...
[INFO] Claude messages request completed successfully
```

## 如果仍然有问题

### 检查 1: 响应预览

在日志中查找 `preview=` 部分，确认内容看起来正常。

### 检查 2: 响应结构

查找 `[DEBUG] Response structure:` 并验证 JSON 格式正确。

### 检查 3: Cline 请求

在日志开始处应该看到 Cline 发送的请求。确认：
- 请求路径是 `/v1/messages`
- 包含正确的认证头
- 消息格式正确

## 使用便捷脚本

如果你经常需要调试，使用这个脚本：

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
./run-kiro-debug.sh
```

这个脚本会：
- 检查配置
- 显示配置信息
- 启动服务器
- 将日志输出到终端和文件

## 文档参考

- **详细调试**: `rust/KIRO_DEBUG_GUIDE.md`
- **使用指南**: `rust/KIRO_USAGE_GUIDE_ZH.md`
- **错误修复**: `CLINE_ERROR_FIX.md`
- **改进总结**: `CLAUDE.md`

## 需要帮助？

如果以上步骤都无法解决问题，请收集以下信息：

1. 运行测试脚本的完整输出：
   ```bash
   ./test_kiro_response.sh your-api-key 8080 > test.log 2>&1
   ```

2. 服务器完整调试日志（发送请求时）：
   ```bash
   RUST_LOG=debug ./target/release/aiclient2api-rust --config config.json 2>&1 | tee server.log
   ```

3. Cline 的完整错误消息（截图）

4. Cline 的配置截图

然后在 GitHub Issues 中报告问题，附上这些信息。

---

**记住**: 最关键的改进是只提取最后一个 assistantResponseMessage 的内容，而不是所有的 content 字段。

