# Thru 产品路线图

> 创建日期: 2026-03-26
> 状态: 设计完成，准备进入 Phase 2

---

## 一、产品定位

```
Thru - AI Agent 友好的跨设备文件传输工具

核心差异化：
┌─────────────────────────────────────────────────────────────┐
│  🎯 AI Agent 友好  │  CLI 一个命令  │  --json 输出          │
├─────────────────────────────────────────────────────────────┤
│  🌐 Tailscale 集成 │  跨网络传输   │  LocalSend 没有       │
├─────────────────────────────────────────────────────────────┤
│  🔀 双协议         │  SSH 远程     │  HTTP 局域网          │
├─────────────────────────────────────────────────────────────┤
│  📱 国内市场       │  鸿蒙适配     │  灵动岛支持           │
└─────────────────────────────────────────────────────────────┘
```

### 与 LocalSend 对比

| 功能 | LocalSend | Thru |
|------|-----------|------|
| 局域网传输 | ✅ HTTP | ✅ HTTP (Phase 3) |
| 跨网络传输 | ❌ | ✅ SSH + Tailscale |
| CLI 支持 | ⚠️ 仅启动参数 | ✅ 完整 CLI |
| AI Agent 友好 | ❌ | ✅ --json 输出 |
| 鸿蒙支持 | ❌ | ✅ Phase 5 |
| 灵动岛 | ❌ | ✅ Phase 5 |
| 设备发现 | UDP 组播 | UDP 组播 + Tailscale |

---

## 二、开发路线图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Thru 开发路线图                           │
├─────────────────────────────────────────────────────────────────┤
│  Phase 1 ✅     │  SSH 基础传输                                  │
│  (已完成)       │  thru send, receive, status, config           │
├─────────────────────────────────────────────────────────────────┤
│  Phase 2        │  SSH 完善                                      │
│  (下一步)       │  ├── 密钥认证配置                              │
│                 │  ├── thru pull（远程拉取）                     │
│                 │  ├── thru send 优化（rsync 替代 scp）          │
│                 │  ├── 降级策略（rsync → scp）                   │
│                 │  └── thru init（初始化向导）                   │
├─────────────────────────────────────────────────────────────────┤
│  Phase 3        │  HTTP 局域网传输（自建）                        │
│                 │  ├── thru serve（电脑端 HTTP 服务）            │
│                 │  ├── thru discover（UDP 组播发现）             │
│                 │  ├── thru send --lan（HTTP 方式）              │
│                 │  ├── 传输优先级（HTTP → rsync → scp）          │
│                 │  └── 手机端：Termux + Python HTTP（临时）      │
├─────────────────────────────────────────────────────────────────┤
│  Phase 4        │  LocalSend 协议兼容                            │
│                 │  ├── 实现完整 LocalSend REST API               │
│                 │  ├── 与 LocalSend App 互传                     │
│                 │  └── PIN 保护 / HTTPS                          │
├─────────────────────────────────────────────────────────────────┤
│  Phase 5        │  多平台 + 灵动岛 + SSH+HTTP 混合               │
│                 │  ├── iOS Dynamic Island                        │
│                 │  ├── 鸿蒙灵动岛                                │
│                 │  ├── 鸿蒙原生 App                              │
│                 │  ├── Flutter GUI（桌面端）                     │
│                 │  └── SSH 隧道 + HTTP 传输（跨网络快速传输）     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、传输方案对比

### rsync vs scp

| 特性 | scp | rsync |
|------|-----|-------|
| 进度显示 | ❌ 无 | ✅ `--progress` |
| 断点续传 | ❌ 不支持 | ✅ 支持 |
| 增量传输 | ❌ 全量 | ✅ 只传变化部分 |
| 压缩 | ❌ 无 | ✅ `-z` 压缩 |
| 速度 | 一般 | 更快 |
| 兼容性 | ✅ 所有系统 | ⚠️ 需要两端安装 |

**结论：rsync 作为主要 SSH 方案，scp 作为兜底。**

### 传输优先级

```
HTTP (局域网)     → 最快，多线程
    ↓ 失败/跨网络
rsync (SSH)       → 有进度，可续传
    ↓ 失败
scp (SSH)         → 最基础，兜底方案
```

### 命令设计

```bash
thru send file.jpg              # 自动选择最佳方案
thru send file.jpg --http       # 强制 HTTP
thru send file.jpg --rsync      # 强制 rsync
thru send file.jpg --scp        # 强制 scp（兜底）
```

---

## 四、Phase 2 详细设计

### 目标
让 SSH 方案完整可用，支持免密登录和远程拉取。

### 功能清单

| 功能 | 命令 | 说明 |
|------|------|------|
| 初始化向导 | `thru init` | 引导用户完成首次配置 |
| 密钥生成 | `thru config keygen` | 生成 SSH 密钥对 |
| 密钥部署 | `thru config key-copy` | 复制公钥到手机 |
| 远程拉取 | `thru pull <file>` | 从手机拉取文件到电脑 |
| | `thru pull --list` | 列出手机上可拉取的文件 |
| 发送优化 | `thru send` | 使用 rsync 替代 scp |
| 进度显示 | `thru send/pull` | 显示传输进度条 |

### 新增命令示例

```bash
# 初始化
thru init
  ? 手机 IP: 100.118.195.54
  ? SSH 端口: 8022
  ? 用户名: u0_a406
  ? 是否配置免密登录? (Y/n)
  ✓ 配置完成！

# 密钥认证
thru config keygen              # 生成密钥对
thru config key-copy            # 一键配置免密登录

# 远程拉取
thru pull photo.jpg             # 拉取单个文件
thru pull --list                # 列出手机 Thru 目录
thru pull --all                 # 拉取全部

# 进度显示
📤 Sending document.pdf...
[████████████████████░░░░] 80%  2.4/3.0 MB
```

---

## 五、Phase 3 详细设计

### 目标
实现 HTTP 局域网传输，提供更快的传输速度。

### 功能清单

| 功能 | 命令 | 说明 |
|------|------|------|
| HTTP 服务 | `thru serve` | 启动 HTTP 文件服务 |
| | `thru serve --port 8080` | 指定端口 |
| 设备发现 | `thru discover` | 发现局域网设备 |
| HTTP 发送 | `thru send --lan` | HTTP 方式发送 |
| 接收文件 | `thru receive` | 接收 HTTP 传输的文件 |

### 手机端临时方案

Phase 3 开发期间，手机端使用 Termux 临时方案：

```bash
# 安装 Python
pkg install python

# 启动 HTTP 服务
python -m http.server 53317 --directory ~/storage/downloads/Thru
```

Phase 5 再开发原生 App。

---

## 六、SSH + HTTP 混合架构（Phase 5）

```
┌─────────────────────────────────────────────────────────────┐
│                    SSH + HTTP 混合架构                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   控制通道 (SSH)              数据通道 (HTTP)                │
│   ┌─────────────┐            ┌─────────────┐                │
│   │ 设备发现    │            │ 文件传输    │                │
│   │ 文件查询    │ ──────────→│ 多线程      │                │
│   │ 权限验证    │            │ 断点续传    │                │
│   │ 远程控制    │            │ 进度显示    │                │
│   └─────────────┘            └─────────────┘                │
│                                                             │
│   优点：安全 + 快速                                         │
│   场景：Tailscale 跨网络时，SSH 做隧道，HTTP 走隧道         │
└─────────────────────────────────────────────────────────────┘
```

### 实现方式

```bash
# SSH 建立 HTTP 隧道
ssh -L 8080:localhost:53317 user@phone

# HTTP 通过隧道传输（快速）
thru send file.jpg --tunnel
```

---

## 七、安全性设计

| 场景 | 方案 | 安全性 |
|------|------|--------|
| 局域网 HTTP | HTTP 明文 | ⚠️ 仅限可信网络 |
| 跨网络 HTTP | SSH 隧道加密 | ✅ 安全 |
| SSH 传输 | SSH 加密 | ✅ 安全 |
| LocalSend 兼容 | HTTPS + PIN | ✅ 安全 |

---

## 八、相关文档

- [CLI 详细设计](../cli-design.md)
- [总体设计](../thru-design.md)
- [Phase 1 实现计划](./2026-03-25-thru-cli-implementation.md)

---

## 九、从 LocalSend 学到的优化

> 详细参考: [LocalSend 技术参考](../references/localsend-reference.md)

### 核心问题修复 (P0)

| 问题 | 位置 | 影响 | 解决方案 | 状态 |
|------|------|------|----------|------|
| HTTP 服务端不保存文件 | http_server.rs:68-77 | 功能缺失 | 实现文件保存 | ✅ 已修复 |
| HTTP 上传无认证 | http_server.rs | 安全风险 | IP 网段白名单 | ✅ 已修复 |

### 架构优化 (P1-P2)

| 问题 | 当前设计 | LocalSend 设计 | 优化 | 状态 |
|------|----------|----------------|------|------|
| 进度条不显示 | HTTP 层更新进度 | 应用层更新进度 | Channel 模式 | ✅ 已修复 |
| 设备 IP 硬编码 | "0.0.0.0" | 获取实际 IP | local_ip_address | ✅ 已修复 |
| 同步阻塞 | std::process::Command | tokio::process::Command | 异步化 | 📅 延后 |
| 无并发上传 | 单线程 | TaskRunner 50 并发 | 添加并发控制 | 📅 延后 |

### 新功能建议

| 功能 | 来源 | 优先级 |
|------|------|--------|
| Token 认证 | LocalSend V3 | P0 |
| HTTP Scan Discovery | LocalSend | P2 |
| 并发上传 | LocalSend TaskRunner | P2 |
| IPv6 双栈 | LocalSend | P3 |

---

## 十、当前状态

- Phase 1: ✅ 已完成
- Phase 2: 📋 准备开始详细设计
- Phase 3-5: 📅 待开发
- **P0 问题**: ✅ 已修复（HTTP 保存 + IP 认证）
- **P1 问题**: ✅ 已修复（进度条 + 设备 IP）