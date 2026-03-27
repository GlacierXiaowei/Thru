# Phase 4: HTTP 完善设计文档

> **创建日期**: 2026-03-27
> **状态**: 设计完成，待实施
> **前置**: Phase 3 ✅ 已完成

---

## 一、目标

完善 Phase 3 的 HTTP 功能，实现：
1. **进度条** - 文件传输时显示进度和速度
2. **自动发现** - 无需手动输入 IP，自动发现并选择设备
3. **降级策略** - HTTP 失败时自动降级到 rsync/scp
4. **receive 命令** - 电脑端主动等待接收文件

---

## 二、批次划分

| Batch | 功能 | 任务数 |
|-------|------|--------|
| **Batch 1** | 进度条 | Task 1-3 |
| **Batch 2** | 自动发现 + 降级策略 | Task 4-6 |
| **Batch 3** | receive 命令 | Task 7-9 |

---

## 三、架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Phase 4 架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Batch 1: 进度条                                             │
│  ┌─────────────────┐              ┌─────────────────┐       │
│  │  HTTP Client    │              │  Progress Bar   │       │
│  │  (reqwest)      │──────────→   │  (indicatif)    │       │
│  └─────────────────┘              └─────────────────┘       │
│                                                             │
│  Batch 2: 自动发现 + 降级                                     │
│  ┌─────────────────┐              ┌─────────────────┐       │
│  │  Auto-Discovery │              │  Fallback Logic │       │
│  │  (UDP → Select) │              │  HTTP→rsync→scp │       │
│  └─────────────────┘              └─────────────────┘       │
│                                                             │
│  Batch 3: receive 命令                                        │
│  ┌─────────────────┐              ┌─────────────────┐       │
│  │  thru receive   │              │  HTTP Server    │       │
│  │  (等待接收)      │←───────────  │  (已有)         │       │
│  └─────────────────┘              └─────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、Batch 1: 进度条

### 4.1 目标

HTTP 发送时显示实时进度条，包括：
- 已传输字节数 / 总字节数
- 传输百分比
- 当前速度
- 剩余时间估算

### 4.2 技术方案

```
┌─────────────────────────────────────────────────────────────┐
│                    进度条实现流程                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. 读取文件大小                                             │
│       ↓                                                     │
│  2. 创建 ProgressBar (indicatif)                            │
│       ↓                                                     │
│  3. 使用 reqwest::Body::wrap_stream() 流式上传               │
│       ↓                                                     │
│  4. 每次读取数据块时更新进度                                  │
│       ↓                                                     │
│  5. 上传完成，显示统计信息                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 进度条样式

```
📤 Sending photo.jpg
[████████████████████░░░░] 80%  2.4/3.0 MB  5.2 MB/s  ETA: 0s
```

### 4.4 新增文件

```
src/utils/progress.rs    # 进度条工具模块
```

### 4.5 修改文件

```
src/core/http_client.rs  # 添加进度回调
src/commands/send.rs     # 使用进度条
```

---

## 五、Batch 2: 自动发现 + 降级策略

### 5.1 自动发现流程

```
thru send file.jpg --lan
       │
       ├─ 1. UDP 组播发现设备 (复用 discovery.rs)
       │      ↓
       ├─ 2. 显示设备列表
       │      ↓
       │   发现的设备：
       │     1. 我的小米15 (100.118.195.54:53317) [Tailscale]
       │     2. MacBook Pro (192.168.1.50:53317) [局域网]
       │      ↓
       ├─ 3. 用户选择设备
       │      ↓
       └─ 4. HTTP POST 发送
```

### 5.2 降级策略

```
HTTP 发送
    │
    ├─ 成功 → 完成
    │
    └─ 失败
         │
         ├─ rsync 可用？ → rsync 发送
         │      │
         │      ├─ 成功 → 完成
         │      │
         │      └─ 失败 → scp 发送
         │
         └─ rsync 不可用 → scp 发送
```

### 5.3 命令示例

```bash
# 自动发现 + 选择设备
thru send file.jpg --lan

# 直接指定 IP（跳过发现）
thru send file.jpg --lan 192.168.1.50:53317

# 强制使用 HTTP（不降级）
thru send file.jpg --lan --no-fallback

# 强制使用 rsync
thru send file.jpg --rsync

# 强制使用 scp
thru send file.jpg --scp
```

### 5.4 修改文件

```
src/commands/send.rs     # 添加交互式设备选择和降级逻辑
src/core/transfer.rs     # 统一传输接口
```

---

## 六、Batch 3: receive 命令

### 6.1 目标

实现电脑端主动等待接收文件，支持手机向电脑发送。

### 6.2 使用场景

```
电脑端                          手机端
───────                        ───────
thru receive                   python thru_phone.py send file.jpg
    │                               │
    │  ←── HTTP POST ────────────── │
    │                               │
收到文件 ✓
```

### 6.3 命令设计

```bash
thru receive                 # 等待接收文件
thru receive --port 8080     # 指定端口
thru receive --json          # JSON 输出
thru receive --save-dir ~/Documents/Thru  # 指定保存目录
```

### 6.4 输出示例

```
📥 等待接收文件...
  保存到: ~/Downloads/Thru/
  按 Ctrl+C 停止

📥 收到文件: photo.jpg (2.4 MB)
✓ 已保存到: ~/Downloads/Thru/photo.jpg

📥 收到文件: document.pdf (1.2 MB)
✓ 已保存到: ~/Downloads/Thru/document.pdf
```

### 6.5 JSON 输出

```json
{
  "event": "file_received",
  "file": {
    "name": "photo.jpg",
    "size": 2411724,
    "path": "~/Downloads/Thru/photo.jpg"
  },
  "timestamp": "2026-03-27T15:30:00+08:00"
}
```

### 6.6 新增文件

```
src/commands/receive.rs   # receive 命令处理
```

### 6.7 修改文件

```
src/main.rs               # 添加 Receive 命令
src/core/http_server.rs   # 添加接收模式
```

---

## 七、新增命令汇总

| 命令 | 说明 |
|------|------|
| `thru receive` | 等待接收文件 |
| `thru receive --port` | 指定端口 |
| `thru receive --json` | JSON 输出 |

---

## 八、修改命令

| 命令 | 变更 |
|------|------|
| `thru send --lan` | 自动发现设备 + 进度条 |
| `thru send --lan <IP:PORT>` | 直接发送 + 进度条 |
| `thru send --no-fallback` | 禁用降级 |
| `thru send --rsync` | 强制 rsync |
| `thru send --scp` | 强制 scp |

---

## 九、项目结构变更

```
src/
├── main.rs                    # 添加 Receive 命令
├── commands/
│   ├── mod.rs                 # 添加 pub mod receive
│   ├── send.rs                # 修改：自动发现 + 降级
│   └── receive.rs             # 新增：接收命令
├── core/
│   ├── http_server.rs         # 修改：添加接收模式
│   ├── http_client.rs         # 修改：添加进度条
│   └── transfer.rs            # 修改：统一传输接口
└── utils/
    ├── mod.rs                 # 添加 pub mod progress
    └── progress.rs            # 新增：进度条工具
```

---

## 十、依赖

无新增依赖，复用现有：
- `indicatif` - 进度条
- `reqwest` - HTTP 客户端
- `tokio` - 异步运行时

---

## 十一、验证清单

- [ ] `thru send file.jpg --lan` 显示进度条
- [ ] `thru send file.jpg --lan` 自动发现设备
- [ ] 多设备时显示选择列表
- [ ] HTTP 失败时自动降级到 rsync
- [ ] rsync 失败时自动降级到 scp
- [ ] `thru receive` 启动接收服务
- [ ] `thru receive --json` JSON 输出
- [ ] 手机发送文件，电脑正常接收

---

## 十二、相关文档

- [Phase 3 HTTP 传输设计](./2026-03-27-phase3-http-transfer.md)
- [Phase 3 实施计划](./2026-03-27-phase3-implementation.md)
- [产品路线图](./2026-03-26-thru-roadmap.md)