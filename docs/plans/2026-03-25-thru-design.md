# Thru 设计文档

> 创建日期: 2026-03-25
> 状态: 设计完成，待开发

---

## 一、项目概述

| 项目 | 内容 |
|------|------|
| **名称** | Thru |
| **定位** | 手机-电脑文件互传工具 |
| **技术栈** | Flutter + Rust（CLI 核心） + Windows OpenSSH Server |
| **开发顺序** | 电脑端 CLI → 电脑端 GUI → 手机端 App |

---

## 二、架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    Thru CLI (Rust)                      │
├─────────────────────────────────────────────────────────┤
│  Commands:                                              │
│  ├─ status    → 检测 SSH/Tailscale/IP                   │
│  ├─ start     → 启动 Windows OpenSSH Server             │
│  ├─ stop      → 停止 Windows OpenSSH Server             │
│  ├─ send      → SCP 发送文件到手机                       │
│  ├─ receive   → 列出/监控已接收文件                       │
│  ├─ list      → 列出 Thru 文件夹                         │
│  ├─ config    → 配置手机 IP                              │
│  └─ help      → 显示帮助                                 │
├─────────────────────────────────────────────────────────┤
│  Core Modules:                                          │
│  ├─ ssh_manager.rs    → SSH Server 控制                  │
│  ├─ tailscale.rs      → Tailscale 状态检测               │
│  ├─ transfer.rs       → SCP 文件传输                     │
│  ├─ config.rs         → 配置管理                         │
│  └─ file_watcher.rs   → 文件夹监控 (--watch)             │
└─────────────────────────────────────────────────────────┘
```

---

## 三、CLI 命令详细设计

```
thru <command> [options]

Commands:
  status                    显示连接状态
    --json                  JSON 格式输出（方便 AI 解析）
  
  start                     启动 SSH Server
    --auto                  自动配置防火墙规则
  
  stop                      停止 SSH Server
  
  send <file>               发送文件到手机
    -d, --dest <path>       指定手机目标路径（默认 Thru/）
    -r, --recursive         发送文件夹
  
  receive                   列出已接收文件
    --watch                 实时监控新文件
    --json                  JSON 格式输出
  
  list                      列出 Thru 文件夹内容
    -a, --all               显示隐藏文件
  
  config                    配置管理
    set-ip <ip>             设置手机 IP
    get-ip                  显示当前手机 IP
    auto-detect             从 Tailscale 自动检测
    show                    显示所有配置
  
  help, --help              显示帮助信息
  version, -v               显示版本
```

### 命令使用示例

```bash
# 查看状态
thru status
thru status --json

# 启动/停止 SSH Server
thru start
thru stop

# 发送文件
thru send document.pdf
thru send ./folder -r

# 接收文件
thru receive
thru receive --watch

# 配置
thru config show
thru config set-ip 100.118.195.54
thru config auto-detect
```

---

## 四、数据存储

| 数据 | 位置 | 格式 |
|------|------|------|
| **配置文件** | `~/.thru/config.toml` | TOML |
| **接收文件** | `~/Downloads/Thru/` | 文件夹 |
| **日志文件** | `~/.thru/thru.log` | 文本 |

### config.toml 示例

```toml
[device]
phone_ip = "100.118.195.54"
phone_user = "u0_a406"
phone_port = 8022

[paths]
receive_dir = "~/Downloads/Thru"

[ssh]
auto_start = true
```

---

## 五、核心模块实现方案

| 模块 | 实现方式 |
|------|---------|
| **SSH Server 控制** | 调用 PowerShell: `Start-Service sshd` / `Stop-Service sshd` |
| **Tailscale 检测** | 调用 `tailscale status --json` 解析输出 |
| **文件传输** | 调用系统 `scp` 命令 |
| **文件监控** | Rust `notify` crate |
| **配置管理** | Rust `toml` crate |

---

## 六、依赖库 (Cargo.toml)

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
chrono = "0.4"
dirs = "5.0"
```

---

## 七、项目结构

```
thru/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   ├── start.rs
│   │   ├── stop.rs
│   │   ├── send.rs
│   │   ├── receive.rs
│   │   ├── list.rs
│   │   └── config.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── ssh_manager.rs
│   │   ├── tailscale.rs
│   │   ├── transfer.rs
│   │   ├── file_watcher.rs
│   │   └── config.rs
│   └── utils/
│       ├── mod.rs
│       └── output.rs
└── README.md
```

---

## 八、开发阶段

| 阶段 | 内容 | 预计时间 |
|------|------|---------|
| **Phase 1** | CLI 基础框架 + status 命令 | 1 天 |
| **Phase 2** | start/stop + config 命令 | 1 天 |
| **Phase 3** | send/receive/list 命令 | 2 天 |
| **Phase 4** | 文件监控 + JSON 输出 | 1 天 |
| **Phase 5** | 测试 + 文档 | 1 天 |

---

## 九、开发路线

### Phase 1: 电脑端 CLI（当前阶段）

- [ ] Rust 项目初始化
- [ ] `thru status` 命令
- [ ] `thru start/stop` 命令
- [ ] `thru config` 命令
- [ ] `thru send/receive/list` 命令
- [ ] 文件监控 `--watch`

### Phase 2: 电脑端 GUI

- [ ] Flutter 桌面项目初始化
- [ ] 状态页面
- [ ] 文件页面
- [ ] 发送页面
- [ ] 调用 Rust CLI 核心逻辑

### Phase 3: 手机端 App

- [ ] Flutter 移动端项目
- [ ] Rust 内嵌 SSH Server (FFI)
- [ ] Android 权限处理
- [ ] 完整功能

---

## 十、相关配置

### 现有 SSH 配置（已配置完成）

| 项目 | 值 |
|------|-----|
| 手机 IP | 100.118.195.54 |
| SSH 端口 | 8022 |
| 用户名 | u0_a406 |
| 认证方式 | SSH 密钥（免密） |
| Tailscale 设备名 | xiaomi-15 |

### Windows OpenSSH Server 配置

```powershell
# 检查状态
Get-Service sshd

# 启动服务
Start-Service sshd

# 停止服务
Stop-Service sshd

# 设置开机自启
Set-Service -Name sshd -StartupType 'Automatic'
```

---

## 十一、备注

- 手机端目前使用 Termux + sshd 方案，已在 2026-03-25 配置完成
- 电脑端使用 Windows 自带 OpenSSH Server
- 文件传输使用 SCP 协议
- 通过 Tailscale 实现内网穿透