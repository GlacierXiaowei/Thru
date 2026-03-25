# Thru CLI 详细设计

> 创建日期: 2026-03-25
> 版本: 0.1.0

---

## 一、命令概览

```
thru <command> [options]

Commands:
  status      显示连接状态
  start       启动 SSH Server
  stop        停止 SSH Server
  send        发送文件到手机
  receive     列出/监控已接收文件
  list        列出 Thru 文件夹
  history     查看传输历史
  config      配置管理

Global Options:
  --help, -h      显示帮助
  --version, -V   显示版本
```

---

## 二、命令详细设计

### 2.1 `thru status`

显示连接状态。

```
thru status
thru status --json
```

**输出格式：**

```
SSH Server:  ● Running
Tailscale:   ● Connected (xiaomi-15)
Phone IP:    100.118.195.54:8022
```

**JSON 输出：**

```json
{
  "ssh_server": { "status": "running", "port": 22 },
  "tailscale": { "status": "connected", "device": "xiaomi-15" },
  "phone": { "ip": "100.118.195.54", "port": 8022, "reachable": true }
}
```

---

### 2.2 `thru start`

启动 SSH Server。

```
thru start
thru start --auto    # 自动配置防火墙规则
```

---

### 2.3 `thru stop`

停止 SSH Server。

```
thru stop
```

---

### 2.4 `thru send`

发送文件到手机。

```
thru send <file>
thru send <file> -d <dest>
thru send <folder> -r
```

| 选项 | 说明 |
|------|------|
| `-d, --dest <path>` | 手机目标路径，默认 `~/storage/downloads/` |
| `-r, --recursive` | 发送文件夹 |

**输出：**

```
📤 Sending photo.jpg...
  → xiaomi-15 (100.118.195.54)
  ✓ Sent 2.3 MB in 1.2s
```

---

### 2.5 `thru receive`

列出/监控已接收文件。

```
thru receive
thru receive --watch
thru receive --json
```

| 选项 | 说明 |
|------|------|
| `--watch` | 实时监控新文件 |
| `--json` | JSON 格式输出 |

**`--watch` 输出格式：**

```
[2026-03-25 14:30:22] 📥 收到文件
  设备: 我的小米15 (xiaomi-15)
  文件: photo.jpg
  大小: 2.3 MB
```

**设备识别逻辑：**
1. 先查 config 别名映射
2. 没有别名 → 调用 `tailscale status --json` 查设备名
3. 都没有 → 直接显示 IP

---

### 2.6 `thru list`

列出 Thru 文件夹内容。

```
thru list
thru list -a
```

| 选项 | 说明 |
|------|------|
| `-a, --all` | 显示隐藏文件 |

---

### 2.7 `thru history`

查看传输历史。

```
thru history
thru history --all
thru history --json
thru history --clear
thru history --keep <n>
```

| 选项 | 说明 |
|------|------|
| `--all` | 显示全部记录 |
| `--json` | JSON 格式输出 |
| `--clear` | 清除所有历史 |
| `--keep <n>` | 只保留最近 n 条 |

---

### 2.8 `thru config`

配置管理。

```
thru config show
thru config set-ip <ip>
thru config set-alias <ip> <name>
thru config get-ip
thru config auto-detect
```

| 子命令 | 说明 |
|--------|------|
| `show` | 显示所有配置 |
| `set-ip <ip>` | 设置手机 IP |
| `set-alias <ip> <name>` | 设置设备别名 |
| `get-ip` | 显示当前手机 IP |
| `auto-detect` | 从 Tailscale 自动检测 |

---

## 三、项目结构

```
thru/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   ├── start.rs
│   │   ├── stop.rs
│   │   ├── send.rs
│   │   ├── receive.rs
│   │   ├── list.rs
│   │   ├── config.rs
│   │   └── history.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── ssh_manager.rs
│   │   ├── tailscale.rs
│   │   ├── transfer.rs
│   │   ├── file_watcher.rs
│   │   ├── config.rs
│   │   └── history.rs
│   └── utils/
│       ├── mod.rs
│       └── output.rs
└── docs/
```

---

## 四、核心模块说明

| 模块 | 职责 | 实现方式 |
|------|------|---------|
| `ssh_manager` | SSH Server 控制 | PowerShell: `Start-Service sshd` |
| `tailscale` | Tailscale 状态检测 | `tailscale status --json` |
| `transfer` | SCP 文件传输 | 调用系统 `scp` 命令 |
| `file_watcher` | 文件夹监控 | `notify` crate |
| `config` | 配置管理 | `toml` crate |
| `history` | 历史记录 | `serde_json` |

---

## 五、依赖库

```toml
[package]
name = "thru"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
notify = "6.1"
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
anyhow = "1.0"
thiserror = "2.0"
```

---

## 六、打包分发

**Phase 1 策略：** 直接分发 exe

```bash
cargo build --release
# 输出: target/release/thru.exe
```

**后续可选：**
- `cargo-wix` 生成 MSI 安装包
- 发布到 Scoop / Chocolatey