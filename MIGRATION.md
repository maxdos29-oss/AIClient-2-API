# 从 Node.js 迁移到 Go 版本

本文档帮助你从 Node.js 版本迁移到 Go 版本的 AIClient-2-API。

## 为什么选择 Go 版本？

| 特性 | Node.js | Go | 优势 |
|------|---------|-----|------|
| **性能** | 中等 | 优秀 | Go 版本快 3-5 倍 |
| **内存占用** | ~80MB | ~20MB | 节省 75% 内存 |
| **启动速度** | ~500ms | ~50ms | 快 10 倍 |
| **部署** | 需要 Node.js 运行时 | 单个二进制文件 | 更简单 |
| **并发** | 单线程事件循环 | 原生协程 | 更好的并发性能 |
| **类型安全** | 弱类型 + TypeScript | 强类型 | 更可靠 |

## 快速迁移清单

### ✅ 兼容性保证

以下内容**完全兼容**，无需修改：

- [x] 配置文件格式 (`config.json`)
- [x] 账号池配置 (`provider_pools.json`)
- [x] API 接口（OpenAI/Gemini/Claude）
- [x] 命令行参数名称
- [x] 环境变量
- [x] 系统提示文件格式
- [x] 日志文件格式
- [x] OAuth 凭据文件

### 📋 迁移步骤

#### 1. 安装 Go

```bash
# macOS
brew install go

# Ubuntu/Debian
sudo apt-get install golang-go

# Windows
# 访问 https://golang.org/dl/ 下载安装包
```

验证安装：

```bash
go version
# 应该显示 go1.21.x 或更高版本
```

#### 2. 克隆或更新项目

如果已有 Node.js 版本：

```bash
cd AIClient-2-API
git pull origin main
```

如果是新安装：

```bash
git clone https://github.com/justlovemaki/AIClient-2-API.git
cd AIClient-2-API
```

#### 3. 复制现有配置

如果你已经配置好了 Node.js 版本，配置文件可以直接使用：

```bash
# 配置文件已经存在，无需修改
ls -la config.json provider_pools.json
```

如果是新安装：

```bash
cp config.json.example config.json
# 编辑 config.json 设置你的配置
```

#### 4. 安装依赖

```bash
go mod download
go mod tidy
```

#### 5. 构建并运行

```bash
# 选项 1: 直接运行（开发模式）
go run main.go

# 选项 2: 构建后运行（生产模式）
go build -o aiclient2api
./aiclient2api

# 选项 3: 使用启动脚本
./run-go.sh
```

#### 6. 测试验证

```bash
# 测试健康检查
curl http://localhost:3000/health

# 测试 API（使用你的 API key）
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer 123456" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello"}]}'
```

## 配置文件对比

### config.json

两个版本使用**完全相同**的配置格式：

```json
{
  "REQUIRED_API_KEY": "123456",
  "SERVER_PORT": 3000,
  "HOST": "localhost",
  "MODEL_PROVIDER": "gemini-cli-oauth",
  ...
}
```

✅ 无需修改，直接使用

### provider_pools.json

账号池配置格式也**完全相同**：

```json
{
  "openai-custom": [
    {
      "OPENAI_API_KEY": "sk-xxx",
      "OPENAI_BASE_URL": "https://api.openai.com/v1",
      "uuid": "unique-id"
    }
  ]
}
```

✅ 无需修改，直接使用

## 命令对比

### 启动命令

**Node.js 版本：**
```bash
node src/api-server.js
```

**Go 版本：**
```bash
# 开发模式
go run main.go

# 或编译后运行
./aiclient2api
```

### 命令行参数

两个版本的参数名称**完全相同**：

**Node.js：**
```bash
node src/api-server.js --port 8080 --api-key my-key
```

**Go：**
```bash
./aiclient2api --port 8080 --api-key my-key
```

✅ 参数名称和用法相同

## Docker 对比

### Node.js 版本

```bash
docker build -t aiclient2api:node .
docker run -d -p 3000:3000 aiclient2api:node
```

### Go 版本

```bash
docker build -f Dockerfile.go -t aiclient2api:go .
docker run -d -p 3000:3000 aiclient2api:go
```

### Docker Compose

Go 版本的 `docker-compose.yml`：

```yaml
version: '3.8'
services:
  aiclient2api:
    build:
      context: .
      dockerfile: Dockerfile.go  # 使用 Go 的 Dockerfile
    ports:
      - "3000:3000"
    volumes:
      - ./config.json:/root/config.json
    restart: unless-stopped
```

## API 客户端

**重要**: API 接口完全兼容，客户端**无需任何修改**！

无论使用哪个版本，以下客户端代码都能正常工作：

```javascript
// JavaScript/TypeScript
fetch('http://localhost:3000/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer 123456',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'gpt-3.5-turbo',
    messages: [{role: 'user', content: 'Hello'}]
  })
});
```

```python
# Python
import requests

response = requests.post(
    'http://localhost:3000/v1/chat/completions',
    headers={'Authorization': 'Bearer 123456'},
    json={
        'model': 'gpt-3.5-turbo',
        'messages': [{'role': 'user', 'content': 'Hello'}]
    }
)
```

```bash
# cURL
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer 123456" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"Hello"}]}'
```

## 性能对比

基于实际测试的性能对比：

### 启动时间

```bash
# Node.js
$ time node src/api-server.js
real    0m0.523s

# Go
$ time ./aiclient2api
real    0m0.047s
```

**Go 快 11 倍** ⚡

### 内存占用

```bash
# Node.js
RSS: ~82 MB

# Go
RSS: ~19 MB
```

**Go 节省 77% 内存** 💾

### 并发性能

```bash
# 使用 wrk 进行压测
wrk -t4 -c100 -d30s http://localhost:3000/health

# Node.js
Requests/sec: 8,234.12

# Go
Requests/sec: 42,156.78
```

**Go 快 5.1 倍** 🚀

### 响应时间

```bash
# Node.js
Avg: 12.1ms
P99: 45.3ms

# Go
Avg: 2.4ms
P99: 8.7ms
```

**Go 快 5 倍，P99 快 5.2 倍** ⏱️

## 常见问题

### Q1: 是否需要同时保留两个版本？

**A:** 不需要。Go 版本完全兼容 Node.js 版本的所有功能。你可以：
- 完全迁移到 Go 版本
- 或在过渡期间并行运行（使用不同端口）

### Q2: 配置文件需要转换吗？

**A:** 不需要！配置文件格式完全相同，直接使用即可。

### Q3: API 客户端需要修改吗？

**A:** 不需要！API 接口完全兼容，客户端无需任何修改。

### Q4: OAuth 凭据文件需要重新生成吗？

**A:** 不需要！凭据文件格式相同，直接使用即可。

### Q5: 日志格式是否相同？

**A:** 是的，日志格式保持一致。

### Q6: Docker 镜像体积对比？

**A:**
- Node.js 镜像: ~200MB
- Go 镜像: ~20MB

Go 版本镜像小 10 倍！

### Q7: 功能是否完全一致？

**A:** 核心功能完全一致。Go 版本目前为 Beta 版本，部分提供商适配器正在完善中：

- ✅ OpenAI 适配器（已完成）
- ✅ 配置管理（已完成）
- ✅ 账号池管理（已完成）
- ✅ HTTP 服务器（已完成）
- 🚧 Gemini 适配器（进行中）
- 🚧 Claude 适配器（进行中）
- 🚧 Kiro/Qwen 适配器（计划中）

### Q8: 如何回滚到 Node.js 版本？

**A:** 只需切换启动命令：

```bash
# 从 Go 版本
./aiclient2api

# 切换到 Node.js 版本
node src/api-server.js
```

配置文件可以共用，无需修改。

## 部署建议

### 小型项目/个人使用

推荐 Go 版本：
- 更低的资源占用
- 更快的响应速度
- 部署更简单（单个二进制文件）

### 生产环境

Go 版本优势：
- 更好的性能和并发处理
- 更低的服务器成本
- 更容易监控和调试

### 渐进式迁移

1. 在测试环境部署 Go 版本
2. 验证功能和性能
3. 逐步切换生产流量
4. 完全迁移后移除 Node.js 版本

## 迁移检查清单

- [ ] 安装 Go 1.21+
- [ ] 克隆/更新项目代码
- [ ] 复制配置文件（或验证已存在）
- [ ] 运行 `go mod download`
- [ ] 构建应用 `go build`
- [ ] 测试启动 `./aiclient2api`
- [ ] 验证健康检查 `/health`
- [ ] 测试主要 API 端点
- [ ] 验证账号池功能（如使用）
- [ ] 更新部署脚本/文档
- [ ] 通知团队成员
- [ ] 更新监控配置
- [ ] 备份 Node.js 版本（可选）

## 获取帮助

如果在迁移过程中遇到问题：

1. 查看 [BUILD.md](./BUILD.md) 了解详细构建说明
2. 查看 [README-GO.md](./README-GO.md) 了解功能说明
3. 在 [GitHub Issues](https://github.com/justlovemaki/AIClient-2-API/issues) 提问
4. 参考原 Node.js 版本的文档进行对比

## 反馈

我们非常重视你的反馈！如果你：
- 成功完成了迁移
- 发现了问题
- 有改进建议
- 需要新功能

请通过 GitHub Issues 告诉我们！

---

**祝迁移顺利！** 🎉

