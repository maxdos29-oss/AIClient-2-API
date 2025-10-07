# 📦 构建产物位置指南

## 🎯 GitHub Actions 构建产物

### 1. **查看构建产物**

访问: https://github.com/maxdos29-oss/AIClient-2-API/actions

1. 点击最新的成功构建（绿色 ✓）
2. 在页面右上角找到 **"Artifacts"** 旁边的数字（例如 "6"）
3. 点击数字 "6" 就能看到所有可下载的构建产物：

```
📦 Artifacts (保留 7 天)
├── aiclient2api-linux-amd64
├── aiclient2api-linux-arm64
├── aiclient2api-darwin-amd64
├── aiclient2api-darwin-arm64
├── aiclient2api-windows-amd64.exe
└── aiclient2api-windows-arm64.exe
```

### 2. **下载构建产物**

点击任意产物名称即可下载对应平台的可执行文件。

**平台说明**：
- `linux-amd64`: Linux x86_64（大多数云服务器）
- `linux-arm64`: Linux ARM64（树莓派等）
- `darwin-amd64`: macOS Intel
- `darwin-arm64`: macOS Apple Silicon (M1/M2/M3)
- `windows-amd64.exe`: Windows x86_64
- `windows-arm64.exe`: Windows ARM64

---

## 🚀 Release 下载

当您推送标签时（如 `v1.0.0`），会自动创建 Release：

```bash
# 创建并推送标签
git tag v1.0.0
git push origin v1.0.0
```

Release 页面: https://github.com/maxdos29-oss/AIClient-2-API/releases

**Release 包含**：
- `.tar.gz` 格式（Linux/macOS）
- `.zip` 格式（Windows）
- 自动生成的更新日志

---

## 💻 本地构建

如果需要本地构建：

```bash
# 克隆仓库
git clone https://github.com/maxdos29-oss/AIClient-2-API.git
cd AIClient-2-API

# 构建当前平台版本
go build -o aiclient2api .

# 或使用提供的脚本构建所有平台
./build-all-platforms.sh
```

本地构建产物位置：
```
./build/
├── aiclient2api-darwin-amd64
├── aiclient2api-darwin-arm64
├── aiclient2api-linux-amd64
├── aiclient2api-linux-arm64
├── aiclient2api-windows-amd64.exe
└── aiclient2api-windows-arm64.exe
```

---

## 📝 使用方法

### Linux/macOS
```bash
# 下载后添加执行权限
chmod +x aiclient2api-linux-amd64

# 运行
./aiclient2api-linux-amd64 -host 0.0.0.0 -port 8080
```

### Windows
```powershell
# 直接运行
.\aiclient2api-windows-amd64.exe -host 0.0.0.0 -port 8080
```

---

## ⚡ 快速获取

**最快方式**：
1. 访问 [Actions 页面](https://github.com/maxdos29-oss/AIClient-2-API/actions)
2. 点击最新的绿色 ✓ 构建
3. 点击页面右上角的数字 "6"（Artifacts）
4. 下载您需要的平台版本

**注意**：Artifacts 会在 7 天后自动删除，如需长期保存，请创建 Release。
