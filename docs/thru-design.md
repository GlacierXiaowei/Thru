# Thru 总设计文档

> 创建日期: 2026-03-25
> 更新日期: 2026-03-25
> 状态: 设计完成，待开发

---

## 一、项目概述

| 项目 | 内容 |
|------|------|
| **名称** | Thru (Through 的缩写) |
| **定位** | 手机-电脑文件互传工具 |
| **技术栈** | Flutter + Rust（核心库） + Windows OpenSSH Server |
| **开发顺序** | 电脑端 CLI → 电脑端 GUI → 手机端 App |

---

## 二、核心场景

**电脑端 UI 主要功能：**
1. **发送文件** → 电脑主动发到手机
2. **监控接收** → 手机发来的文件实时显示

---

## 三、总体架构

```
┌─────────────────────────────────────────────────────────┐
│                    thru-lib (Rust)                      │
│  ┌─────────────┬─────────────┬─────────────┬──────────┐ │
│  │ ssh_manager │  tailscale  │  transfer   │  config  │ │
│  └─────────────┴─────────────┴─────────────┴──────────┘ │
│  ┌─────────────┬─────────────┐                          │
│  │file_watcher │   history   │                          │
│  └─────────────┴─────────────┘                          │
└─────────────────────────────────────────────────────────┘
         ▲              ▲              ▲
         │              │              │
    ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
    │ thru-cli│    │ Flutter │    │ Flutter │
    │ (Rust)  │    │  GUI    │    │ Mobile  │
    └─────────┘    └────────┘    └─────────┘
```

### 渐进式开发策略

| 阶段 | 结构 | 理由 |
|------|------|------|
| **Phase 1** | 单 crate（bin + 内部模块） | 快速开发，先跑起来 |
| **Phase 2 前** | 拆分为 `thru-lib` + `thru-cli` | 准备给 GUI 用 |
| **Phase 2/3** | 添加 FFI 层 | Flutter 集成 |

---

## 四、数据存储

| 数据 | 位置 | 格式 |
|------|------|------|
| **配置文件** | `~/.thru/config.toml` | TOML |
| **接收文件** | `~/Downloads/Thru/` | 文件夹 |
| **历史记录** | `~/.thru/history.json` | JSON |
| **日志文件** | `~/.thru/thru.log` | 文本 |

> `~` 表示用户主目录（Windows: `C:\Users\<用户名>\`）

---

## 五、配置文件结构

**位置：** `~/.thru/config.toml`

```toml
[device]
phone_ip = "100.118.195.54"
phone_user = "u0_a406"
phone_port = 8022

[aliases]
"100.118.195.54" = "我的小米15"

[paths]
receive_dir = "~/Downloads/Thru"

[ssh]
auto_start = true
```

---

## 六、历史记录格式

**位置：** `~/.thru/history.json`

```json
[
  {
    "id": 1,
    "type": "receive",
    "timestamp": "2026-03-25T14:30:22+08:00",
    "device": {
      "name": "xiaomi-15",
      "alias": "我的小米15",
      "ip": "100.118.195.54"
    },
    "file": {
      "name": "photo.jpg",
      "size": 2411724,
      "path": "~/Downloads/Thru/photo.jpg"
    },
    "status": "completed"
  }
]
```

---

## 七、开发阶段

| 阶段 | 内容 | 状态 |
|------|------|------|
| **Phase 1** | CLI 完整功能 | 待开发 |
| **Phase 2** | 电脑端 GUI (Flutter) | 未开始 |
| **Phase 3** | 手机端 App (Flutter + Rust FFI) | 未开始 |

---

## 八、相关文档

- [CLI 详细设计](./cli-design.md)
- [开发计划](./plans/)

---

## 九、现有 SSH 配置

| 项目 | 值 |
|------|-----|
| 手机 IP | 100.118.195.54 |
| SSH 端口 | 8022 |
| 用户名 | u0_a406 |
| 认证方式 | SSH 密钥（免密） |
| Tailscale 设备名 | xiaomi-15 |