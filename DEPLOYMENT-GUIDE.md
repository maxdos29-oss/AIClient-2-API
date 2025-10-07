# GitHub Actions 部署指南

## 🎯 概述

本项目已配置完整的 GitHub Actions CI/CD 工作流，可自动构建多平台版本并发布。

## 📋 工作流说明

### 1. Build and Release (`.github/workflows/build.yml`)

**触发条件**:
- Push 到 `main` 或 `master` 分支
- 创建 `v*` 标签 (如 `v0.9.0`)
- Pull Request 到 `main` 或 `master`

**构建平台**:
| 平台 | 架构 | 输出文件 |
|------|------|----------|
| Linux | amd64 | `aiclient2api-linux-amd64` |
| Linux | arm64 | `aiclient2api-linux-arm64` |
| macOS | amd64 | `aiclient2api-darwin-amd64` |
| macOS | arm64 | `aiclient2api-darwin-arm64` |
| Windows | amd64 | `aiclient2api-windows-amd64.exe` |
| Windows | arm64 | `aiclient2api-windows-arm64.exe` |

**Docker 镜像**:
- 自动构建多架构镜像 (linux/amd64, linux/arm64)
- 推送到 Docker Hub (需配置 secrets)
- 标签策略: `latest`, `v0.9.0`, `v0.9`, SHA, 分支名

**Release**:
- 标签推送时自动创建 GitHub Release
- 上传所有平台的二进制文件 (tar.gz/zip)
- 自动生成 Release Notes

### 2. Lint (`.github/workflows/lint.yml`)

**检查项**:
- ✅ golangci-lint 代码质量检查
- ✅ gofmt 格式检查
- ✅ go mod tidy 依赖检查

### 3. Security Scan (`.github/workflows/security.yml`)

**安全扫描**:
- ✅ Gosec - Go 安全扫描
- ✅ Trivy - 漏洞扫描
- 结果上传到 GitHub Security

**定时执行**: 每周日自动运行

## 🔧 配置 GitHub Secrets

### Docker Hub 发布 (可选)

在仓库设置中添加以下 Secrets:

```
Settings -> Secrets and variables -> Actions -> New repository secret
```

**必需的 Secrets**:
- `DOCKER_USERNAME`: Docker Hub 用户名
- `DOCKER_PASSWORD`: Docker Hub 访问令牌

### 获取 Docker Hub 令牌

1. 登录 [Docker Hub](https://hub.docker.com/)
2. 进入 Account Settings -> Security
3. 点击 "New Access Token"
4. 复制生成的令牌并添加到 GitHub Secrets

## 🚀 发布新版本

### 方法 1: 使用标签发布

```bash
# 1. 确保所有更改已提交
git add .
git commit -m "feat: prepare for release"
git push origin main

# 2. 创建版本标签
git tag -a v0.9.0 -m "Release version 0.9.0"

# 3. 推送标签到远程（触发构建和发布）
git push origin v0.9.0
```

### 方法 2: 通过 GitHub 界面

1. 进入仓库页面
2. 点击 "Releases" -> "Draft a new release"
3. 点击 "Choose a tag" -> 输入新标签 (如 `v0.9.0`)
4. 填写 Release 标题和描述
5. 点击 "Publish release"

GitHub Actions 会自动:
- ✅ 构建所有平台版本
- ✅ 构建 Docker 镜像
- ✅ 上传二进制文件到 Release
- ✅ 发布 Docker 镜像到 Docker Hub

## 📦 下载构建产物

### GitHub Releases

访问: `https://github.com/YOUR_USERNAME/AIClient-2-API/releases`

下载对应平台的文件:
- Linux: `.tar.gz`
- macOS: `.tar.gz`
- Windows: `.zip`

### Docker 镜像

```bash
# 拉取最新版本
docker pull YOUR_USERNAME/aiclient2api:latest

# 拉取特定版本
docker pull YOUR_USERNAME/aiclient2api:v0.9.0

# 运行
docker run -d -p 3000:3000 YOUR_USERNAME/aiclient2api:latest
```

### Artifacts (临时构建)

在 Actions 页面:
1. 点击对应的 workflow run
2. 下载 "Artifacts" 区域的文件
3. 有效期: 7 天

## 🔍 监控构建状态

### 查看工作流状态

访问: `https://github.com/YOUR_USERNAME/AIClient-2-API/actions`

### 添加徽章到 README

```markdown
[![Build](https://github.com/YOUR_USERNAME/AIClient-2-API/workflows/Build%20and%20Release/badge.svg)](https://github.com/YOUR_USERNAME/AIClient-2-API/actions)
[![Lint](https://github.com/YOUR_USERNAME/AIClient-2-API/workflows/Lint/badge.svg)](https://github.com/YOUR_USERNAME/AIClient-2-API/actions)
[![Security](https://github.com/YOUR_USERNAME/AIClient-2-API/workflows/Security%20Scan/badge.svg)](https://github.com/YOUR_USERNAME/AIClient-2-API/actions)
```

## 🐛 故障排除

### 构建失败

1. **检查日志**: 点击失败的 workflow -> 查看详细日志
2. **常见问题**:
   - Go 版本不匹配
   - 依赖下载失败
   - 测试失败
   - 格式检查失败

### Docker 推送失败

1. 检查 Secrets 配置是否正确
2. 确认 Docker Hub 令牌有效
3. 检查镜像名称是否正确

### Release 创建失败

1. 确认标签格式正确 (以 `v` 开头)
2. 检查是否有权限创建 Release
3. 确认 `GITHUB_TOKEN` 有足够权限

## 📊 构建统计

### 构建时间 (预估)

| 任务 | 时间 |
|------|------|
| 单平台构建 | ~2-3 分钟 |
| 所有平台构建 | ~15-20 分钟 |
| Docker 构建 | ~5-10 分钟 |
| 总计 | ~20-30 分钟 |

### 资源消耗

- GitHub Actions 免费额度: 2000 分钟/月 (公开仓库无限)
- Artifact 存储: 最多 7 天
- Docker Hub: 免费账户有拉取限制

## 🎓 最佳实践

### 1. 版本号规范

遵循语义化版本 [Semantic Versioning](https://semver.org/):

```
v<major>.<minor>.<patch>

例如:
v1.0.0 - 首个稳定版
v1.1.0 - 添加新功能
v1.1.1 - Bug 修复
v2.0.0 - 重大更新
```

### 2. Release Notes

创建 Release 时提供:
- ✅ 新功能列表
- ✅ Bug 修复
- ✅ 破坏性变更
- ✅ 升级说明
- ✅ 已知问题

### 3. 测试策略

发布前:
1. ✅ 本地测试通过
2. ✅ CI 测试通过
3. ✅ 手动测试核心功能
4. ✅ 检查文档更新

### 4. 发布节奏

建议:
- 🔄 主分支: 持续集成
- 📅 Beta 版本: 每周或每两周
- 🎯 稳定版本: 每月
- 🚨 Hotfix: 根据需要

## 🔐 安全建议

1. **保护分支**
   - 在 Settings -> Branches 设置保护规则
   - 要求 PR review
   - 要求状态检查通过

2. **Secrets 管理**
   - 定期轮换 tokens
   - 使用最小权限原则
   - 不要在代码中硬编码密钥

3. **依赖安全**
   - 定期运行安全扫描
   - 更新依赖到最新版本
   - 关注安全公告

## 📞 获取帮助

### GitHub Actions 文档
- [官方文档](https://docs.github.com/en/actions)
- [工作流语法](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)

### 问题反馈
- [项目 Issues](https://github.com/justlovemaki/AIClient-2-API/issues)
- [GitHub Discussions](https://github.com/justlovemaki/AIClient-2-API/discussions)

---

**祝部署顺利！** 🚀

