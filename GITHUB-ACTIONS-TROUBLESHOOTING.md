# GitHub Actions 故障排查指南

## ❌ 常见错误及解决方案

### 错误 1: "账户因账单问题被锁定"

**错误信息**:
```
The job was not started because your account is locked due to a billing issue.
```

**原因**: GitHub 账户存在付款问题

**解决方法**:

1. **检查账单状态**
   - 访问: https://github.com/settings/billing
   - 查看是否有未付款或过期的付款方式

2. **更新付款信息**
   - Settings → Billing and plans → Payment information
   - 添加或更新信用卡

3. **使用免费额度**（公开仓库）
   - 公开仓库有免费的 Actions 额度
   - Settings → 确保仓库是 Public

4. **本地构建（临时方案）**
   ```bash
   # 使用本地脚本构建
   ./build-all-platforms.sh
   ```

### 错误 2: 依赖下载失败

**解决方法**:
```yaml
- name: Download dependencies
  run: |
    go mod download
    go mod verify
```

### 错误 3: 构建失败

**解决方法**:
- 检查代码是否能在本地编译
- 运行: `go build -v .`

## 🔧 本地构建方案

由于您的 GitHub 账户存在账单问题，建议使用本地构建：

### 使用本地构建脚本

```bash
# 1. 运行多平台构建脚本
./build-all-platforms.sh

# 2. 查看构建结果
ls -lh build/

# 3. 测试运行
./build/aiclient2api-linux-amd64  # Linux
# 或
./build/aiclient2api-darwin-arm64  # macOS Apple Silicon
```

### 手动构建单个平台

```bash
# Linux
CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o aiclient2api-linux-amd64

# macOS (Apple Silicon)
CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build -o aiclient2api-darwin-arm64

# Windows
CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -o aiclient2api-windows.exe
```

## 📝 临时禁用 GitHub Actions

如果暂时不需要 Actions，可以禁用：

### 方法 1: 仓库设置

1. Settings → Actions → General
2. 选择 "Disable actions"

### 方法 2: 删除 workflow 文件

```bash
# 临时移除（不推荐）
git mv .github/workflows .github/workflows.disabled
git commit -m "chore: temporarily disable GitHub Actions"
git push
```

## 🎯 推荐方案

### 短期方案（立即可用）

✅ **使用本地构建脚本**
```bash
./build-all-platforms.sh
```

这样您可以:
- ✅ 构建所有平台版本
- ✅ 生成 .tar.gz 和 .zip 文件
- ✅ 无需 GitHub Actions
- ✅ 完全控制构建过程

### 长期方案

1. **解决 GitHub 账单问题**
   - 更新付款信息
   - 等待账户解锁

2. **或使用其他 CI/CD**
   - GitLab CI (有免费额度)
   - CircleCI (有免费额度)
   - 本地 Jenkins

## 💡 替代方案

### 使用 GitLab CI

创建 `.gitlab-ci.yml`:

```yaml
image: golang:1.21

stages:
  - build

build:
  stage: build
  script:
    - go mod download
    - CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o aiclient2api-linux
  artifacts:
    paths:
      - aiclient2api-linux
```

### 使用本地 Docker 构建

```bash
# 构建 Docker 镜像
docker build -f Dockerfile.go -t aiclient2api:local .

# 运行
docker run -d -p 3000:3000 aiclient2api:local
```

## 📞 获取帮助

1. **GitHub 支持**: https://support.github.com/
2. **账单问题**: https://github.com/settings/billing/payment_information
3. **Actions 文档**: https://docs.github.com/actions

---

**总结**: 由于账单问题，请使用 `./build-all-platforms.sh` 在本地构建，或解决 GitHub 账单问题后重新运行 Actions。

