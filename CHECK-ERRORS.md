# GitHub Actions 错误检查指南

## 🔍 如何查看 GitHub Actions 错误

### 方法 1: GitHub 网页查看

1. **访问 Actions 页面**
   ```
   https://github.com/maxdos28/AIClient-2-API/actions
   ```

2. **点击失败的 workflow run** (红色 ❌ 标记)

3. **查看具体的 job 日志**
   - 点击失败的 job (如 "Build")
   - 展开失败的 step
   - 查看详细错误信息

### 方法 2: 使用 GitHub CLI

```bash
# 安装 GitHub CLI (如果未安装)
brew install gh  # macOS
# 或
apt-get install gh  # Ubuntu

# 登录
gh auth login

# 查看最近的 workflow runs
gh run list

# 查看特定 run 的日志
gh run view <run-id> --log
```

---

## 🐛 常见的 Go 编译错误

### 1. 导入未使用的包

**错误示例**:
```
imported and not used: "bufio"
imported and not used: "regexp"
```

**原因**: 导入了包但没有使用

**修复**: 删除未使用的 import

### 2. 变量声明但未使用

**错误示例**:
```
declared but not used: ctx
```

**修复**: 删除未使用的变量或使用 `_` 忽略

### 3. 类型不匹配

**错误示例**:
```
cannot use ... (type string) as type []byte
```

**修复**: 添加类型转换

### 4. 缺少依赖

**错误示例**:
```
no required module provides package github.com/...
```

**修复**:
```bash
go get github.com/package/name
go mod tidy
```

---

## 🔧 本地检查方法

### 快速检查

```bash
# 1. 下载依赖
go mod download

# 2. 检查语法
go vet ./...

# 3. 检查格式
gofmt -l .

# 4. 尝试编译
go build -v .
```

### 详细检查

```bash
# 运行测试构建脚本
./test-build.sh
```

---

## 🔨 可能的修复

基于我看到的代码，可能的问题：

### 问题 1: 未使用的 import

**文件**: 可能多个文件

**检查**:
```bash
go build -v . 2>&1 | grep "imported and not used"
```

### 问题 2: go.sum 缺失

**修复**:
```bash
go mod tidy
```

### 问题 3: 依赖版本问题

**修复**:
```bash
go get -u github.com/google/uuid@latest
go mod tidy
```

---

## 📋 检查清单

请在 GitHub Actions 页面查看错误后，告诉我具体的错误信息：

- [ ] 是编译错误吗？
- [ ] 是依赖下载错误吗？
- [ ] 是测试失败吗？
- [ ] 是权限问题吗？
- [ ] 具体的错误消息是什么？

---

## 💡 临时解决方案

如果 GitHub Actions 持续失败，可以：

### 方案 1: 使用本地构建

```bash
./build-all-platforms.sh
```

### 方案 2: 禁用测试 job

修改 `.github/workflows/build.yml`，注释掉 test job

### 方案 3: 简化 workflow

只保留基本的构建步骤

---

**请提供 GitHub Actions 的具体错误信息，我将帮您修复！** 🔧

