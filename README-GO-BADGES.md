# README 徽章

将以下徽章添加到您的 README-GO.md 文件中，展示项目状态。

## 🏷️ 推荐徽章

### 基础信息

```markdown
[![Go Version](https://img.shields.io/github/go-mod/go-version/justlovemaki/AIClient-2-API)](https://golang.org/)
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Release](https://img.shields.io/github/v/release/justlovemaki/AIClient-2-API)](https://github.com/justlovemaki/AIClient-2-API/releases)
```

### CI/CD 状态

```markdown
[![Build](https://github.com/justlovemaki/AIClient-2-API/workflows/Build%20and%20Release/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)
[![Lint](https://github.com/justlovemaki/AIClient-2-API/workflows/Lint/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)
[![Security](https://github.com/justlovemaki/AIClient-2-API/workflows/Security%20Scan/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)
```

### 代码质量

```markdown
[![Go Report Card](https://goreportcard.com/badge/github.com/justlovemaki/AIClient-2-API)](https://goreportcard.com/report/github.com/justlovemaki/AIClient-2-API)
[![codecov](https://codecov.io/gh/justlovemaki/AIClient-2-API/branch/main/graph/badge.svg)](https://codecov.io/gh/justlovemaki/AIClient-2-API)
```

### Docker

```markdown
[![Docker Image Size](https://img.shields.io/docker/image-size/YOUR_USERNAME/aiclient2api/latest)](https://hub.docker.com/r/YOUR_USERNAME/aiclient2api)
[![Docker Pulls](https://img.shields.io/docker/pulls/YOUR_USERNAME/aiclient2api)](https://hub.docker.com/r/YOUR_USERNAME/aiclient2api)
```

### 社区

```markdown
[![GitHub stars](https://img.shields.io/github/stars/justlovemaki/AIClient-2-API?style=social)](https://github.com/justlovemaki/AIClient-2-API)
[![GitHub forks](https://img.shields.io/github/forks/justlovemaki/AIClient-2-API?style=social)](https://github.com/justlovemaki/AIClient-2-API)
[![GitHub issues](https://img.shields.io/github/issues/justlovemaki/AIClient-2-API)](https://github.com/justlovemaki/AIClient-2-API/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
```

### 性能指标

```markdown
[![Startup Time](https://img.shields.io/badge/Startup-50ms-green)](README-GO.md#performance)
[![Memory Usage](https://img.shields.io/badge/Memory-20MB-green)](README-GO.md#performance)
[![Concurrency](https://img.shields.io/badge/Concurrency-5000%20req%2Fs-green)](README-GO.md#performance)
```

## 📝 完整示例

```markdown
<div align="center">

![logo](src/img/logo-min.webp)

# AIClient-2-API (Go Version) 🚀

[![Go Version](https://img.shields.io/github/go-mod/go-version/justlovemaki/AIClient-2-API)](https://golang.org/)
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Release](https://img.shields.io/github/v/release/justlovemaki/AIClient-2-API)](https://github.com/justlovemaki/AIClient-2-API/releases)

[![Build](https://github.com/justlovemaki/AIClient-2-API/workflows/Build%20and%20Release/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)
[![Lint](https://github.com/justlovemaki/AIClient-2-API/workflows/Lint/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)
[![Security](https://github.com/justlovemaki/AIClient-2-API/workflows/Security%20Scan/badge.svg)](https://github.com/justlovemaki/AIClient-2-API/actions)

[![Startup Time](https://img.shields.io/badge/Startup-50ms-green)](README-GO.md#performance)
[![Memory Usage](https://img.shields.io/badge/Memory-20MB-green)](README-GO.md#performance)
[![Concurrency](https://img.shields.io/badge/Concurrency-5000%20req%2Fs-green)](README-GO.md#performance)

**一个能将多种仅客户端内使用的大模型 API，统一封装为本地 OpenAI 兼容接口的高性能 Go 代理。**

**10x 更快 • 4x 更少内存 • 单个二进制**

[快速开始](QUICKSTART-GO.md) | [构建指南](BUILD.md) | [迁移指南](MIGRATION.md) | [完整文档](README-GO.md)

</div>
```

## 🎨 自定义徽章

### 创建自定义徽章

使用 [Shields.io](https://shields.io/) 创建:

```markdown
![Custom](https://img.shields.io/badge/Status-Production%20Ready-success)
![Custom](https://img.shields.io/badge/Completion-90%25-yellow)
![Custom](https://img.shields.io/badge/Platform-Multi-blue)
```

### 动态徽章

```markdown
<!-- 最后提交时间 -->
![Last Commit](https://img.shields.io/github/last-commit/justlovemaki/AIClient-2-API)

<!-- 代码大小 -->
![Code Size](https://img.shields.io/github/languages/code-size/justlovemaki/AIClient-2-API)

<!-- 仓库大小 -->
![Repo Size](https://img.shields.io/github/repo-size/justlovemaki/AIClient-2-API)

<!-- 语言分布 -->
![Languages](https://img.shields.io/github/languages/top/justlovemaki/AIClient-2-API)

<!-- Contributors -->
![Contributors](https://img.shields.io/github/contributors/justlovemaki/AIClient-2-API)
```

## 📊 状态页面

### GitHub Actions 状态

查看所有工作流状态:
```
https://github.com/justlovemaki/AIClient-2-API/actions
```

### Docker Hub 状态

查看 Docker 镜像:
```
https://hub.docker.com/r/YOUR_USERNAME/aiclient2api
```

## 🔔 通知设置

### 设置 Actions 通知

1. 进入仓库 Settings
2. 选择 Notifications
3. 配置 Actions 通知:
   - ✅ 失败时通知
   - ✅ 首次成功通知
   - ⬜ 每次成功通知 (可选)

### Slack/Discord 集成

在 workflow 中添加通知步骤:

```yaml
- name: Notify on Slack
  if: always()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

## 📈 性能监控

### 构建性能

查看 Actions 页面的:
- ⏱️ 构建时间趋势
- 📊 成功率
- 🔄 重试次数

### 优化建议

1. **使用缓存**: 已配置 Go modules 缓存
2. **并行构建**: 使用 matrix 策略
3. **条件执行**: 避免不必要的任务

## 🎯 下一步

1. ✅ 配置 GitHub Secrets (Docker Hub)
2. ✅ 添加徽章到 README
3. ✅ 创建第一个 Release
4. ✅ 测试自动构建
5. ✅ 监控构建状态

---

**GitHub Actions 已配置完成！推送标签即可触发自动构建和发布。** 🎉

