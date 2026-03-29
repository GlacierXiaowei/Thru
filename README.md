# Thru

AI Agent 友好的跨设备文件传输工具，支持 SSH 和 HTTP 两种方式实现手机与电脑之间的文件互传。

## 快速开始

```bash
# 方式 1: HTTP 局域网传输（推荐，有进度条）
thru serve                          # 电脑启动接收服务
thru send 文件路径 --lan            # 发送到手机

# 方式 2: SSH 传输（备用）
thru init --ip <手机 IP> --user <Termux 用户名>
thru config keygen
thru config key-copy   # 按提示在手机执行命令
thru send 文件路径
```

## 手机端配置

1. 安装 [Termux](https://termux.com/)
2. 运行以下命令：
   ```bash
   pkg install openssh rsync
   sshd
   whoami  # 查看用户名
   ifconfig  # 查看IP
   ```

## 命令列表

| 命令 | 说明 |
|------|------|
| `thru intro` | 快速入门指南 |
| `thru init` | 配置手机信息 (SSH) |
| `thru serve` | 启动 HTTP 接收服务 |
| `thru send <file>` | 发送文件到手机 |
| `thru send <file> --lan` | HTTP 局域网发送 (有进度条) |
| `thru send <file> --lan <IP>` | 指定 IP 发送 |
| `thru pull <file>` | 从手机拉取文件 |
| `thru pull --list` | 列出手机上的文件 |
| `thru pull --all` | 拉取全部文件 |
| `thru status` | 查看连接状态 |
| `thru list` | 列出已接收文件 |
| `thru history` | 查看传输历史 |
| `thru config keygen` | 生成 SSH 密钥 |
| `thru config key-copy` | 部署公钥到手机 |

## 高级选项

```bash
# 传输方式选择
thru send file.jpg --lan            # HTTP 局域网传输（推荐，有进度条）
thru send file.jpg --lan <IP>       # 指定 IP 发送
thru send file.jpg --rsync          # SSH rsync（备用）
thru send file.jpg --scp            # SSH scp（备用）

# JSON 输出（适合脚本调用）
thru status --json
thru list --json
thru history --json
thru pull --list --json
thru send file.jpg --json
```

## 常见问题

### Q: 发送大文件时连接中断？
A: 可能是后台进程冲突，先清理：
```bash
taskkill /F /IM thru.exe
thru serve  # 重新启动服务
```

### Q: CMD 中看不到进度条？
A: 进度条使用 `\r` 实现单行更新，CMD 中可能显示多行历史，这是正常的。最终会显示 `✓ 发送完成`。

### Q: Tailscale 异地传输不稳定？
A: 异地传输网络波动是正常的，HTTP 失败会自动降级 SCP。建议局域网内使用。

### Q: 手机如何接收文件？
A: 手机需要运行 Python 接收服务端 (`thru_phone_server.py`)，或使用 Termux 的 SSH 模式。

## 开发进度

- [x] Phase 1: 基础 CLI 实现
- [x] Phase 2: SSH 完善
  - [x] Batch 1: 密钥管理
  - [x] Batch 2: init 命令 + rsync 支持
  - [x] Batch 3: pull 命令 + json 支持
- [x] Phase 3: HTTP 局域网传输
  - [x] Batch 1: HTTP 服务器（`thru serve`）
  - [x] Batch 2: 设备发现（`thru discover`）
  - [x] Batch 3: HTTP 发送（`thru send --lan`）
- [x] Phase 4: HTTP 优化
  - [x] 文件保存到 ~/Downloads/Thru/
  - [x] IP 网段白名单认证
  - [x] 进度条显示（单行更新）
  - [x] 自动检测设备 IP
- [ ] Phase 5: LocalSend 协议兼容
- [ ] Phase 6: Flutter GUI + 手机端 App

详细设计文档见 `docs/` 目录。