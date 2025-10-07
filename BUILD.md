# 构建和安装指南

## 前置要求

### 安装 Go

本项目需要 Go 1.21 或更高版本。

#### macOS

```bash
# 使用 Homebrew
brew install go

# 或下载安装包
# 访问 https://golang.org/dl/ 下载 macOS 版本
```

#### Linux

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install golang-go

# 或使用官方脚本
wget https://golang.org/dl/go1.21.0.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
export PATH=$PATH:/usr/local/go/bin
```

#### Windows

访问 https://golang.org/dl/ 下载并安装 Windows 版本的 Go。

### 验证安装

```bash
go version
# 应该输出: go version go1.21.x ...
```

## 构建步骤

### 1. 克隆项目

```bash
git clone https://github.com/justlovemaki/AIClient-2-API.git
cd AIClient-2-API
```

### 2. 安装依赖

```bash
go mod download
```

### 3. 配置文件

```bash
# 复制示例配置文件
cp config.json.example config.json
cp provider_pools.json provider_pools.example.json

# 编辑 config.json 设置你的配置
# 根据需要编辑 provider_pools.json
```

### 4. 构建

#### 开发模式 (直接运行)

```bash
go run main.go
```

#### 生产构建

```bash
# 构建当前平台的二进制文件
go build -o aiclient2api

# 构建并优化大小
go build -ldflags="-s -w" -o aiclient2api

# 运行
./aiclient2api
```

#### 交叉编译

```bash
# Linux
GOOS=linux GOARCH=amd64 go build -o aiclient2api-linux-amd64

# Windows
GOOS=windows GOARCH=amd64 go build -o aiclient2api-windows-amd64.exe

# macOS (Apple Silicon)
GOOS=darwin GOARCH=arm64 go build -o aiclient2api-darwin-arm64

# macOS (Intel)
GOOS=darwin GOARCH=amd64 go build -o aiclient2api-darwin-amd64
```

## Docker 构建

### 构建镜像

```bash
docker build -f Dockerfile.golang -t aiclient2api:go .
```

### 运行容器

```bash
# 基本运行
docker run -d \
  --name aiclient2api \
  -p 3000:3000 \
  aiclient2api:go

# 挂载配置文件
docker run -d \
  --name aiclient2api \
  -p 3000:3000 \
  -v $(pwd)/config.json:/root/config.json \
  -v $(pwd)/provider_pools.json:/root/provider_pools.json \
  aiclient2api:go

# 使用环境变量
docker run -d \
  --name aiclient2api \
  -p 3000:3000 \
  -e REQUIRED_API_KEY=your-api-key \
  -e MODEL_PROVIDER=openai-custom \
  -e OPENAI_API_KEY=sk-xxx \
  aiclient2api:go

# 使用 Docker Compose
docker-compose up -d
```

### Docker Compose 配置

创建 `docker-compose.yml`:

```yaml
version: '3.8'

services:
  aiclient2api:
    build:
      context: .
      dockerfile: Dockerfile.golang
    container_name: aiclient2api
    ports:
      - "3000:3000"
    volumes:
      - ./config.json:/root/config.json
      - ./provider_pools.json:/root/provider_pools.json
    environment:
      - REQUIRED_API_KEY=123456
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
```

## 开发调试

### 启用详细日志

```bash
go run main.go --log-prompts console
```

### 使用 Air 进行热重载

```bash
# 安装 Air
go install github.com/cosmtrek/air@latest

# 运行
air
```

创建 `.air.toml`:

```toml
root = "."
tmp_dir = "tmp"

[build]
  bin = "./tmp/main"
  cmd = "go build -o ./tmp/main ."
  delay = 1000
  exclude_dir = ["tmp", "vendor"]
  exclude_regex = ["_test.go"]
  include_ext = ["go", "json"]
  kill_delay = "0s"
  log = "build-errors.log"
  send_interrupt = false
  stop_on_error = true

[color]
  app = ""
  build = "yellow"
  main = "magenta"
  runner = "green"
  watcher = "cyan"

[log]
  time = false

[misc]
  clean_on_exit = true
```

## 性能优化

### 构建优化

```bash
# 禁用符号表和调试信息，减小二进制大小
go build -ldflags="-s -w" -o aiclient2api

# 使用 UPX 压缩（可选）
upx --best --lzma aiclient2api
```

### 运行时优化

```bash
# 设置 GOMAXPROCS（通常无需设置，Go 会自动检测）
export GOMAXPROCS=4

# 启用性能分析
go run main.go -cpuprofile=cpu.prof -memprofile=mem.prof
```

## 测试

### 运行所有测试

```bash
go test ./...
```

### 运行特定包的测试

```bash
go test ./internal/adapter/
go test ./internal/server/
```

### 生成测试覆盖率报告

```bash
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

### 基准测试

```bash
go test -bench=. ./...
```

## 故障排除

### 依赖问题

```bash
# 清理模块缓存
go clean -modcache

# 重新下载依赖
go mod download

# 验证依赖
go mod verify

# 整理依赖
go mod tidy
```

### 构建问题

```bash
# 清理构建缓存
go clean -cache

# 详细构建输出
go build -v

# 检查构建环境
go env
```

### 运行时问题

```bash
# 启用 race detector
go run -race main.go

# 启用详细日志
go run main.go --log-prompts console

# 检查健康状态
curl http://localhost:3000/health
```

## 部署建议

### 生产环境

1. 使用编译后的二进制文件，不要使用 `go run`
2. 配置 systemd 服务（Linux）
3. 使用反向代理（Nginx/Caddy）处理 HTTPS
4. 配置日志轮转
5. 监控健康检查端点
6. 使用配置文件而非环境变量（更安全）

### Systemd 服务示例

创建 `/etc/systemd/system/aiclient2api.service`:

```ini
[Unit]
Description=AIClient-2-API Service
After=network.target

[Service]
Type=simple
User=aiclient2api
WorkingDirectory=/opt/aiclient2api
ExecStart=/opt/aiclient2api/aiclient2api
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启用和启动服务:

```bash
sudo systemctl daemon-reload
sudo systemctl enable aiclient2api
sudo systemctl start aiclient2api
sudo systemctl status aiclient2api
```

### Nginx 反向代理示例

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 更多资源

- [Go 官方文档](https://golang.org/doc/)
- [Go 模块参考](https://go.dev/ref/mod)
- [项目 GitHub](https://github.com/justlovemaki/AIClient-2-API)
- [问题反馈](https://github.com/justlovemaki/AIClient-2-API/issues)

