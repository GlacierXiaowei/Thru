# Thru

AI Agent 友好的跨设备文件传输工具，基于 SSH 实现手机与电脑之间的文件互传。

## 快速开始

```bash
# 1. 配置手机信息
thru init --ip <手机IP> --user <Termux用户名>

# 2. 生成 SSH 密钥（免密登录）
thru config keygen
thru config key-copy   # 按提示在手机执行命令

# 3. 发送文件到手机
thru send 文件路径

# 4. 从手机拉取文件
thru pull --list       # 查看手机上的文件
thru pull 文件名       # 拉取文件
thru pull --all        # 拉取全部文件
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
| `thru init` | 配置手机信息 |
| `thru send <file>` | 发送文件到手机 |
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
thru send file.jpg --rsync    # 强制使用 rsync（带进度条）
thru send file.jpg --scp      # 强制使用 scp

# JSON 输出（适合脚本调用）
thru status --json
thru list --json
thru history --json
thru pull --list --json
thru send file.jpg --json
```

## 开发进度

- [x] Phase 1: 基础 CLI 实现
- [x] Phase 2: SSH 完善
  - [x] Batch 1: 密钥管理
  - [x] Batch 2: init 命令 + rsync 支持
  - [x] Batch 3: pull 命令 + json 支持
- [ ] Phase 3: 高级功能

详细设计文档见 `docs/` 目录。