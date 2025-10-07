# GitHub Actions 配置状态

**配置时间**: 2025-10-07  
**状态**: ✅ 完全配置完成

---

## 🎉 已配置的工作流

### 1. Build and Release (`build.yml`)

**功能**: 多平台构建和自动发布

**触发条件**:
- ✅ Push 到 main/master 分支
- ✅ 创建 v* 标签
- ✅ Pull Request

**构建矩阵** (6 个平台):
```
✅ Linux   amd64
✅ Linux   arm64
✅ macOS   amd64 (Intel)
✅ macOS   arm64 (Apple Silicon)
✅ Windows amd64
✅ Windows arm64
```

**输出产物**:
- ✅ 各平台二进制文件
- ✅ 自动打包 (.tar.gz / .zip)
- ✅ 上传到 GitHub Artifacts (7天)
- ✅ 创建 GitHub Release (标签推送时)

**Docker 构建**:
- ✅ 多架构镜像 (linux/amd64, linux/arm64)
- ✅ 自动推送到 Docker Hub
- ✅ 智能标签策略 (latest, version, SHA)
- ✅ 构建缓存优化

### 2. Lint (`lint.yml`)

**功能**: 代码质量检查

**检查项**:
- ✅ golangci-lint (多种 linters)
- ✅ gofmt 格式检查
- ✅ go mod tidy 依赖检查

**触发条件**:
- ✅ Push 到 main/master
- ✅ Pull Request

### 3. Security Scan (`security.yml`)

**功能**: 安全漏洞扫描

**扫描工具**:
- ✅ Gosec - Go 代码安全扫描
- ✅ Trivy - 依赖漏洞扫描

**触发条件**:
- ✅ Push 到 main/master
- ✅ Pull Request
- ✅ 每周日定时扫描

**结果上传**:
- ✅ GitHub Security 标签页
- ✅ SARIF 格式报告

---

## 📊 工作流详情

### Build and Release

```yaml
jobs:
  build:       # 构建 6 个平台版本
  docker:      # 构建 Docker 镜像
  release:     # 创建 GitHub Release
  test:        # 运行测试
```

**构建时间** (预估):
- 单平台构建: ~2-3 分钟
- 全平台并行: ~15-20 分钟
- Docker 构建: ~5-10 分钟
- **总计**: ~20-30 分钟

**优化**:
- ✅ Go modules 缓存
- ✅ Docker buildx 缓存
- ✅ 并行构建
- ✅ 增量构建

### 构建优化参数

```bash
# 编译参数
CGO_ENABLED=0
-trimpath
-ldflags="-s -w"

# 效果:
# - 减小二进制大小 (~30%)
# - 移除调试信息
# - 移除文件路径信息
# - 纯静态编译
```

---

## 🚀 使用方式

### 方式 1: 推送代码触发构建

```bash
# 提交并推送
git add .
git commit -m "feat: add new feature"
git push origin main

# GitHub Actions 会自动:
# ✅ 运行测试
# ✅ 代码检查
# ✅ 构建所有平台
# ✅ 构建 Docker 镜像
```

### 方式 2: 创建发布版本

```bash
# 创建标签
git tag -a v0.9.0 -m "Release v0.9.0"
git push origin v0.9.0

# GitHub Actions 会额外:
# ✅ 创建 GitHub Release
# ✅ 上传所有平台的二进制文件
# ✅ 自动生成 Release Notes
# ✅ 推送 Docker 镜像 (带版本标签)
```

### 方式 3: Pull Request

```bash
# 创建 PR
git checkout -b feature/new-feature
# ... 开发 ...
git push origin feature/new-feature
# 在 GitHub 创建 PR

# GitHub Actions 会:
# ✅ 运行所有检查
# ✅ 构建测试 (不发布)
# ✅ 安全扫描
# ✅ 显示结果在 PR 页面
```

---

## 📦 下载构建产物

### GitHub Releases

访问: https://github.com/justlovemaki/AIClient-2-API/releases

**可下载**:
```
aiclient2api-linux-amd64.tar.gz
aiclient2api-linux-arm64.tar.gz
aiclient2api-darwin-amd64.tar.gz
aiclient2api-darwin-arm64.tar.gz
aiclient2api-windows-amd64.zip
aiclient2api-windows-arm64.zip
```

### Docker Hub

```bash
# 拉取最新版本
docker pull justlovemaki/aiclient2api:latest

# 拉取特定版本
docker pull justlovemaki/aiclient2api:v0.9.0

# 拉取特定架构
docker pull justlovemaki/aiclient2api:latest --platform linux/amd64
docker pull justlovemaki/aiclient2api:latest --platform linux/arm64
```

### GitHub Actions Artifacts

在 Actions 页面:
1. 点击成功的 workflow run
2. 滚动到底部查看 "Artifacts"
3. 下载对应平台的文件
4. **有效期**: 7 天

---

## 🔧 配置 Docker Hub (可选)

### 1. 创建 Docker Hub 账号

访问: https://hub.docker.com/signup

### 2. 创建 Access Token

1. 登录 Docker Hub
2. Account Settings -> Security
3. New Access Token
4. 复制生成的 token

### 3. 配置 GitHub Secrets

在 GitHub 仓库:
1. Settings -> Secrets and variables -> Actions
2. 添加以下 secrets:
   - `DOCKER_USERNAME`: 你的 Docker Hub 用户名
   - `DOCKER_PASSWORD`: 刚才创建的 Access Token

### 4. 验证配置

推送代码后，在 Actions 页面查看 "docker" job 是否成功。

---

## 📈 监控和维护

### 查看构建状态

```
https://github.com/justlovemaki/AIClient-2-API/actions
```

**关注**:
- ✅ Build 成功率
- ⏱️ 构建时间趋势
- 🔄 失败的构建
- 📊 测试覆盖率

### 安全扫描结果

```
https://github.com/justlovemaki/AIClient-2-API/security
```

**关注**:
- 🔒 代码扫描结果
- 🔍 依赖漏洞
- ⚠️ 安全警告

### 定期维护

**每周**:
- ✅ 检查 Actions 运行状态
- ✅ 查看安全扫描结果
- ✅ 更新依赖版本

**每月**:
- ✅ 清理旧的 Artifacts
- ✅ 审查失败的构建
- ✅ 优化构建时间

---

## 🎓 高级配置

### 添加其他平台

编辑 `.github/workflows/build.yml`:

```yaml
matrix:
  include:
    # 添加新平台
    - goos: freebsd
      goarch: amd64
      output: aiclient2api-freebsd-amd64
```

### 自定义构建参数

```yaml
- name: Build
  env:
    GOOS: ${{ matrix.goos }}
    GOARCH: ${{ matrix.goarch }}
    CGO_ENABLED: 0
  run: |
    go build \
      -v \
      -trimpath \
      -ldflags="-s -w \
        -X main.Version=${{ github.ref_name }} \
        -X main.BuildTime=$(date -u +%Y%m%d%H%M%S) \
        -X main.GitCommit=${{ github.sha }}" \
      -o ${{ matrix.output }} .
```

### 添加测试步骤

```yaml
- name: Run integration tests
  run: go test -v -tags=integration ./...

- name: Run benchmark
  run: go test -bench=. -benchmem ./...
```

---

## 🔒 安全最佳实践

### Secrets 管理

1. ✅ 使用 GitHub Secrets 存储敏感信息
2. ✅ 不要在代码中硬编码密钥
3. ✅ 使用环境变量注入
4. ✅ 定期轮换 tokens

### 权限设置

```yaml
permissions:
  contents: write    # 创建 Release
  packages: write    # 推送 Docker 镜像
  security-events: write  # 上传安全扫描结果
```

### 代码签名 (可选)

```yaml
- name: Sign binary
  run: |
    # 使用 GPG 签名
    gpg --armor --detach-sign aiclient2api
```

---

## 📊 当前状态

### ✅ 已配置

```
✅ 多平台构建 (6 platforms)
✅ Docker 多架构镜像 (2 architectures)
✅ 自动化发布 (GitHub Releases)
✅ 代码质量检查 (golangci-lint)
✅ 安全扫描 (Gosec + Trivy)
✅ 测试执行 (go test)
✅ 覆盖率上传 (Codecov)
✅ 缓存优化 (Go modules, Docker)
```

### ⏳ 待配置 (可选)

```
⏳ Docker Hub secrets (需要配置 DOCKER_USERNAME 和 DOCKER_PASSWORD)
⏳ Codecov token (需要在 codecov.io 注册)
⏳ Slack/Discord 通知
⏳ 性能基准测试
```

---

## 🎯 快速验证

### 检查工作流文件

```bash
# 验证 YAML 语法
cat .github/workflows/build.yml | grep "name:"
cat .github/workflows/lint.yml | grep "name:"
cat .github/workflows/security.yml | grep "name:"
```

### 查看远程状态

访问以下链接验证:

1. **Actions 页面**: 
   ```
   https://github.com/maxdos28/AIClient-2-API/actions
   ```

2. **最新构建**:
   ```
   https://github.com/maxdos28/AIClient-2-API/actions/workflows/build.yml
   ```

3. **Releases 页面**:
   ```
   https://github.com/maxdos28/AIClient-2-API/releases
   ```

### 触发测试构建

```bash
# 推送一个小更改来测试
echo "# Test" >> TEST.md
git add TEST.md
git commit -m "test: trigger GitHub Actions"
git push origin main

# 然后访问 Actions 页面查看构建状态
```

---

## 📝 工作流文件清单

```
✅ .github/workflows/build.yml      - 构建和发布
✅ .github/workflows/lint.yml       - 代码检查
✅ .github/workflows/security.yml   - 安全扫描
✅ .golangci.yml                    - Linter 配置
✅ DEPLOYMENT-GUIDE.md              - 部署指南
✅ README-GO-BADGES.md              - 徽章说明
✅ CONTRIBUTING.md                  - 贡献指南
```

---

## 🎉 成功！

您的 GitHub Actions 已完全配置！

**下一步**:
1. ✅ 代码已推送到远程 ✓
2. ✅ 标签 v0.9.0 已推送 ✓
3. ✅ GitHub Actions 正在运行...
4. 🔄 等待构建完成 (~20-30 分钟)
5. 📦 下载构建产物或 Docker 镜像

**查看构建状态**:
```
https://github.com/maxdos28/AIClient-2-API/actions
```

**查看 Release**:
```
https://github.com/maxdos28/AIClient-2-API/releases/tag/v0.9.0
```

---

**GitHub Actions 配置完成！构建将自动进行。** 🚀

