# Phase 3: HTTP 局域网传输设计文档

> **创建日期**: 2026-03-27
> **状态**: 设计完成，待实施
> **前置**: Phase 2 ✅ 已完成

---

## 一、目标

实现 **HTTP 局域网传输**，速度比 SSH 快 10-100 倍（多线程 + 无加密开销）。

---

## 二、核心功能

| 功能 | 命令 | 说明 |
|------|------|------|
| HTTP 服务 | `thru serve` | 启动 HTTP 文件服务器 |
| 设备发现 | `thru discover` | UDP 组播发现局域网设备 |
| HTTP 发送 | `thru send --lan` | HTTP 方式发送文件 |
| 端口降级 | 自动 | 53317 → 53318 → 8080 |

---

## 三、技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Phase 3 架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  电脑端 (Rust)                     手机端 (Python 临时)      │
│  ┌─────────────────┐              ┌─────────────────┐       │
│  │  HTTP Server     │              │  http.server     │       │
│  │  (axum + tokio) │              │  (临时验证)      │       │
│  │  端口：53317/18/8080│ ← HTTP →   │  端口：53317     │       │
│  └─────────────────┘              └─────────────────┘       │
│          ▲                              ▲                   │
│          │                              │                   │
│  ┌───────┴────────┐              ┌───────┴────────┐        │
│  │  UDP 组播发现   │ ←───────────→ │  UDP 组播响应   │        │
│  │  (239.12.34.56) │              │                 │        │
│  └────────────────┘              └─────────────────┘        │
│                                                             │
│  传输优先级：HTTP (局域网) → rsync (SSH) → scp (兜底)        │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、端口策略

**自动降级逻辑：**

```
1. 尝试绑定 53317 (LocalSend 兼容端口)
   ↓ 失败（端口被占用）
2. 尝试绑定 53318 (备用端口 1)
   ↓ 失败
3. 尝试绑定 8080 (备用端口 2)
   ↓ 失败
4. 报错：无可用端口
```

**用户提示：**

```
⚠ 端口 53317 已被占用，正在使用备用端口 53318...
  或
⚠ 端口 53317/53318 均不可用，正在使用备用端口 8080...
```

**理由：**
- 53317：Phase 4 兼容 LocalSend App
- 53318：Phase 3 专用，避免冲突
- 8080：通用 HTTP 端口，最后兜底

---

## 五、新增命令

### 5.1 `thru serve`

启动 HTTP 文件服务器。

```bash
thru serve                    # 自动选择端口
thru serve --port 53318       # 指定端口
thru serve --lan              # 仅局域网模式（绑定 0.0.0.0）
```

**输出：**

```
🌐 HTTP 服务已启动
  地址: http://192.168.1.100:53318
  局域网: http://100.118.195.54:53318 (Tailscale)
  
等待接收文件...
按 Ctrl+C 停止
```

**API 端点：**

| 端点 | 方法 | 说明 |
|------|------|------|
| `/` | GET | 服务信息 |
| `/upload` | POST | 接收文件 |
| `/files` | GET | 列出已接收文件 |
| `/device` | GET | 设备信息（用于发现） |

---

### 5.2 `thru discover`

发现局域网设备。

```bash
thru discover              # 搜索 5 秒
thru discover --timeout 10 # 搜索 10 秒
thru discover --json       # JSON 输出
```

**输出：**

```
🔍 正在搜索局域网设备...

发现的设备：
  1. 我的小米15 (100.118.195.54:53317) [Tailscale]
  2. MacBook Pro (192.168.1.50:53317) [局域网]
  
共发现 2 台设备
```

**JSON 输出：**

```json
{
  "devices": [
    {
      "name": "我的小米15",
      "ip": "100.118.195.54",
      "port": 53317,
      "network": "tailscale"
    }
  ],
  "total": 1
}
```

---

### 5.3 `thru send --lan`

HTTP 方式发送文件。

```bash
thru send file.jpg --lan              # 自动发现设备
thru send file.jpg --lan --ip 192.168.1.50  # 指定 IP
```

**自动发现流程：**

```
1. UDP 组播发现设备
2. 显示设备列表供用户选择
3. HTTP POST 发送文件
4. 显示进度条
5. 失败时降级到 rsync/scp
```

---

## 六、设备发现协议

**UDP 组播地址：** `239.12.34.56:53317`

**发现流程：**

```
电脑端 (discover)                     手机端 (serve)
      │                                    │
      │ ──── UDP 组播 "THRU_DISCOVER" ────→ │
      │                                    │
      │ ←─── UDP 单播 "THRU_RESPONSE" ──── │
      │      {name, ip, port, device_id}   │
      │                                    │
```

**消息格式：**

```json
// 发现请求
{
  "type": "THRU_DISCOVER",
  "version": "1.0"
}

// 发现响应
{
  "type": "THRU_RESPONSE",
  "name": "我的小米15",
  "ip": "100.118.195.54",
  "port": 53317,
  "device_id": "uuid-xxx-xxx",
  "network": "tailscale"
}
```

---

## 七、HTTP 文件传输

**上传流程：**

```
┌─────────────────┐                    ┌─────────────────┐
│  电脑 (send)     │                    │  手机 (serve)    │
└────────┬────────┘                    └────────┬────────┘
         │                                      │
         │  POST /upload                        │
         │  Content-Type: multipart/form-data   │
         │  ─────────────────────────────────→  │
         │                                      │
         │          200 OK / 201 Created        │
         │  ←─────────────────────────────────  │
         │          {"success": true}           │
         │                                      │
```

**请求示例：**

```http
POST /upload HTTP/1.1
Host: 100.118.195.54:53317
Content-Type: multipart/form-data; boundary=----ThruBoundary

------ThruBoundary
Content-Disposition: form-data; name="file"; filename="photo.jpg"
Content-Type: image/jpeg

<binary data>
------ThruBoundary--
```

---

## 八、传输优先级

**自动选择逻辑：**

```
thru send file.jpg
    │
    ├─ 局域网环境？ ──→ HTTP (最快)
    │      ↓ 否
    │   rsync 可用？ ──→ rsync (有进度)
    │      ↓ 否
    │   scp (兜底)
    │
thru send file.jpg --lan  → 强制 HTTP
thru send file.jpg --rsync → 强制 rsync
thru send file.jpg --scp   → 强制 scp
```

---

## 九、手机端临时方案

**Phase 3 使用 Python http.server：**

```bash
# 安装 Python
pkg install python

# 启动 HTTP 服务
python -m http.server 53317 --directory ~/storage/downloads/Thru

# 或使用 Thru 提供的脚本（Phase 3 后期）
thru-serve-phone
```

**Phase 5 替换为 Flutter 原生 App。**

---

## 十、安全性

**Phase 3：** 无 PIN 保护（用户主动选择设备，风险可控）

**Phase 4：** 实现设备唯一标识符 + PIN 配对

```
首次连接：
  1. 发现设备 → 显示设备 ID
  2. 输入 PIN（手机端显示）
  3. 配对成功 → 记录信任关系

后续连接：
  1. 发现设备 → 检查信任关系
  2. 已信任 → 直接连接
  3. 未信任 → 要求 PIN
```

**LocalSend 的安全风险：**
- 默认无 PIN，同局域网任何人可发送
- Thru Phase 4 将通过 PIN 配对改善此问题

---

## 十一、新增依赖

```toml
[dependencies]
# 现有依赖
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
indicatif = "0.17"
anyhow = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"

# 新增依赖
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
reqwest = { version = "0.12", features = ["multipart", "stream"] }
uuid = { version = "1.0", features = ["v4"] }
```

---

## 十二、项目结构变更

```
src/
├── main.rs                      # 添加 Serve, Discover 命令
├── commands/
│   ├── mod.rs                   # 添加 pub mod serve, discover
│   ├── serve.rs                 # 新增：HTTP 服务命令
│   ├── discover.rs              # 新增：设备发现命令
│   ├── send.rs                  # 修改：添加 --lan 参数
│   └── ...
├── core/
│   ├── mod.rs                   # 添加 pub mod http, discovery
│   ├── http_server.rs           # 新增：HTTP 服务器模块
│   ├── discovery.rs             # 新增：UDP 组播发现模块
│   ├── http_client.rs           # 新增：HTTP 客户端模块
│   ├── transfer.rs              # 修改：添加 HTTP 传输方式
│   └── ...
└── utils/
    └── output.rs                # 现有
```

---

## 十三、实现批次

| Batch | 任务数 | 内容 |
|-------|--------|------|
| **Batch 1** | Task 1-4 | HTTP 服务器基础（serve 命令） |
| **Batch 2** | Task 5-7 | UDP 组播设备发现（discover 命令） |
| **Batch 3** | Task 8-10 | HTTP 发送集成 + 自动降级 + 测试 |

---

## 十四、与 Phase 4 的关系

| 特性 | Phase 3 | Phase 4 |
|------|---------|---------|
| 协议 | Thru 私有 | LocalSend 兼容 |
| 端口 | 53317/18/8080 | 53317 |
| 发现 | UDP 组播 | LocalSend 协议 |
| 安全 | 无 PIN | PIN + 设备 ID |
| 兼容 | 仅 Thru | LocalSend App |

---

## 十五、相关文档

- [产品路线图](./2026-03-26-thru-roadmap.md)
- [Phase 2 实施计划](./2026-03-26-phase2-implementation.md)
- [CLI 详细设计](../cli-design.md)
- [总体设计](../thru-design.md)