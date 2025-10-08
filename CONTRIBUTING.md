# 贡献指南 / Contributing Guide

感谢您对 AIClient-2-API Go 版本的关注！我们欢迎各种形式的贡献。

## 📋 贡献方式

### 1. 报告问题 (Bug Report)

如果您发现了 bug，请：
1. 检查 [Issues](https://github.com/justlovemaki/AIClient-2-API/issues) 中是否已有相同问题
2. 如果没有，创建新 issue，包含：
   - 问题描述
   - 复现步骤
   - 期望行为
   - 实际行为
   - 环境信息（OS、Go 版本等）
   - 相关日志

### 2. 提出功能建议 (Feature Request)

如果您有好的想法：
1. 创建 issue 并标记为 `enhancement`
2. 详细描述功能需求和使用场景
3. 说明为什么这个功能有用

### 3. 提交代码 (Pull Request)

#### 开发流程

1. **Fork 项目**
   ```bash
   # Fork 后克隆到本地
   git clone https://github.com/YOUR_USERNAME/AIClient-2-API.git
   cd AIClient-2-API
   ```

2. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

3. **开发和测试**
   ```bash
   # 安装依赖
   go mod download
   
   # 运行测试
   go test ./...
   
   # 检查格式
   gofmt -w .
   
   # 运行 linter
   golangci-lint run
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "feat: add new feature" # 或 "fix: fix bug"
   ```

5. **推送并创建 PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   然后在 GitHub 上创建 Pull Request

#### Commit 规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型 (type)**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**:
```
feat(adapter): add Claude streaming support

- Implement SSE streaming for Claude adapter
- Add error handling for stream interruption
- Update tests

Closes #123
```

## 🎯 开发优先级

### 高优先级
- [ ] 完善 Claude 适配器
- [ ] 完善 Kiro 适配器
- [ ] 完善 Qwen 适配器
- [ ] 添加单元测试
- [ ] 添加集成测试

### 中优先级
- [ ] 性能优化
- [ ] 文档改进
- [ ] 示例代码
- [ ] 错误处理增强

### 低优先级
- [ ] UI 界面
- [ ] 监控面板
- [ ] 更多提供商支持

## 📝 代码规范

### Go 代码规范

1. **遵循 Go 官方规范**
   - 使用 `gofmt` 格式化代码
   - 使用 `golangci-lint` 检查代码质量
   - 遵循 [Effective Go](https://golang.org/doc/effective_go.html)

2. **命名规范**
   - 包名：小写，单个单词，如 `adapter`, `server`
   - 接口：名词或形容词，如 `ApiServiceAdapter`
   - 函数：动词开头，如 `GenerateContent`
   - 变量：驼峰命名，如 `configFile`

3. **注释规范**
   - 导出的函数、类型、常量必须有注释
   - 注释以名称开头，如 `// GenerateContent generates...`
   - 复杂逻辑添加行内注释

4. **错误处理**
   ```go
   // ✅ 好的做法
   if err != nil {
       return fmt.Errorf("failed to do something: %w", err)
   }
   
   // ❌ 不好的做法
   if err != nil {
       panic(err)
   }
   ```

5. **并发安全**
   - 使用 `sync.RWMutex` 保护共享状态
   - 避免数据竞争
   - 使用 `go test -race` 检测

### 项目结构

```
AIClient-2-API/
├── main.go              # 主入口
├── internal/            # 内部包
│   ├── common/         # 通用功能
│   ├── adapter/        # API 适配器
│   ├── converter/      # 数据转换
│   ├── pool/           # 账号池
│   └── server/         # HTTP 服务器
├── docs/               # 文档
└── tests/              # 测试
```

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
go test ./...

# 运行特定包的测试
go test ./internal/adapter/

# 运行带覆盖率的测试
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out

# 运行竞态检测
go test -race ./...
```

### 编写测试

```go
func TestGenerateContent(t *testing.T) {
    adapter := NewOpenAIAdapter(&common.Config{
        OpenAIAPIKey: "test-key",
    })
    
    result, err := adapter.GenerateContent("gpt-3.5-turbo", map[string]interface{}{
        "messages": []interface{}{
            map[string]interface{}{
                "role": "user",
                "content": "Hello",
            },
        },
    })
    
    if err != nil {
        t.Fatalf("Expected no error, got %v", err)
    }
    
    if result == nil {
        t.Fatal("Expected result, got nil")
    }
}
```

## 📖 文档

### 更新文档

如果您的更改影响到：
- API 接口
- 配置选项
- 使用方式
- 部署流程

请同时更新相关文档：
- `README-GO.md` - 主要功能文档
- `QUICKSTART-GO.md` - 快速开始指南
- `BUILD.md` - 构建指南
- `MIGRATION.md` - 迁移指南

## 🔍 代码审查

PR 会经过以下检查：
- ✅ CI 构建通过
- ✅ 所有测试通过
- ✅ 代码格式正确
- ✅ Linter 检查通过
- ✅ 无明显的安全问题
- ✅ 代码审查通过

## 💬 交流

- 💬 [GitHub Discussions](https://github.com/justlovemaki/AIClient-2-API/discussions) - 一般讨论
- 🐛 [GitHub Issues](https://github.com/justlovemaki/AIClient-2-API/issues) - Bug 报告和功能请求
- 📧 Email: 通过 issue 联系

## 📜 许可证

提交代码即表示您同意：
- 您的贡献将采用项目的 GPL-3.0 许可证
- 您拥有提交代码的权利
- 您的贡献是原创的或有权使用的

## 🙏 致谢

感谢所有贡献者！您的贡献让这个项目变得更好。

---

**祝您贡献愉快！** 🎉

