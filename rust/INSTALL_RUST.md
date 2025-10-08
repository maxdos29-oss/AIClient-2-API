# 安装 Rust 环境

## 检查是否已安装

```bash
rustc --version
cargo --version
```

如果能看到版本号，说明已安装，可以跳过此步骤。

---

## macOS 安装 Rust

### 方法 1: 官方安装脚本（推荐）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装过程中选择：
1. 选择 `1) Proceed with installation (default)`
2. 等待安装完成

### 方法 2: 使用 Homebrew

```bash
brew install rust
```

---

## 配置环境

### 添加到 PATH

安装完成后，运行：

```bash
source $HOME/.cargo/env
```

或者添加到你的 shell 配置文件（`~/.zshrc` 或 `~/.bashrc`）：

```bash
echo 'source $HOME/.cargo/env' >> ~/.zshrc
source ~/.zshrc
```

---

## 验证安装

```bash
rustc --version
cargo --version
rustup --version
```

应该看到类似：
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
rustup 1.26.0 (5af9b9484 2023-04-05)
```

---

## 更新 Rust

```bash
rustup update stable
```

---

## 卸载 Rust（如果需要）

```bash
rustup self uninstall
```

---

## 安装完成后

### 1. 构建项目

```bash
cd /Users/xuzhaokun/jianshen/AIClient-2-API/rust
cargo build --release
```

首次构建会下载所有依赖，可能需要几分钟。

### 2. 运行 Kiro 提供商

#### 使用配置文件

```bash
cp config-kiro.json config.json
./target/release/aiclient2api-rust
```

#### 使用命令行参数

```bash
./target/release/aiclient2api-rust \
  --host 0.0.0.0 \
  --model-provider claude-kiro-oauth \
  --kiro-oauth-creds-file /Users/xuzhaokun/.aws/sso/cache/kiro-auth-token.json
```

#### 使用启动脚本

```bash
./run-kiro.sh
```

---

## 快速对比

| 步骤 | Node.js | Rust |
|------|---------|------|
| **安装** | `npm install` | `cargo build` |
| **启动** | `node src/api-server.js --...` | `./target/release/aiclient2api-rust --...` |
| **内存** | ~80MB | ~20MB |
| **启动** | ~200ms | ~50ms |

---

## 下一步

安装完成后，查看 [RUN_KIRO.md](./RUN_KIRO.md) 了解如何运行 Kiro 提供商。

