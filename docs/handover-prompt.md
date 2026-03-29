# Thru 项目交接提示词

复制以下内容到新对话：

---

## 项目背景

Thru 是一个 Rust CLI 手机-电脑文件互传工具，支持 SSH/rsync 和 HTTP 局域网传输。

## 当前状态

- Phase 1 ✅ SSH 基础传输
- Phase 2 ✅ SSH 增强（rsync、pull）
- Phase 3 ✅ HTTP 局域网传输（功能实现，但有问题）
- Phase 4 进行中（HTTP 优化）

## 已完成的实现计划

详见：`docs/plans/2026-03-29-http-optimization-plan.md`

该计划包含 4 个问题的完整实现：
- P0-1: HTTP 服务端保存文件
- P0-2: IP 网段认证
- P1-1: Channel 模式进度条
- P1-2: 自动检测设备 IP

## 关键设计决策

**IP 认证方案**：IP 网段白名单
- 允许：192.168.x.x, 10.x.x.x, 172.16-31.x.x (局域网)
- 允许：100.64-127.x.x (Tailscale CGNAT)
- 允许：127.x.x.x (本地测试)
- PIN 配对机制：延后到 Phase 5（手机端开发时）

**进度条架构**：Channel 模式
- 应用层：读取文件 → 更新进度 → 发送 chunk 到 channel
- HTTP 层：接收 channel stream → 发送到网络
- 参考：LocalSend v2.rs 的 ReceiverStream 模式

**新增依赖**：
```toml
tokio-stream = "0.1"
local-ip-address = "0.6"
futures-util = "0.3"
```

## 待修复问题（按优先级）

| 优先级 | 问题 | 位置 | 说明 | 状态 |
|--------|------|------|------|------|
| **P0** | HTTP 服务端不保存文件 | http_server.rs | 保存到 ~/Downloads/Thru/ | 📋 待实现 |
| **P0** | HTTP 上传无认证 | http_server.rs | IP 网段白名单 | 📋 待实现 |
| **P1** | 进度条架构错误 | http_client.rs | Channel 模式 | 📋 待实现 |
| **P1** | 设备 IP 硬编码 | discovery.rs:82 | local_ip_address | 📋 待实现 |
| **P2** | 同步 I/O 阻塞 | transfer.rs | tokio::process | 📅 延后 |
| **P2** | 无并发上传 | 全局 | TaskRunner | 📅 延后 |

## 参考文档

- 设计文档: `docs/thru-design.md`
- 路线图: `docs/plans/2026-03-26-thru-roadmap.md`
- LocalSend 参考: `docs/references/localsend-reference.md`
- **实现计划**: `docs/plans/2026-03-29-http-optimization-plan.md` ⭐

## 执行建议

```
请使用 executing-plans skill 执行实现计划：
docs/plans/2026-03-29-http-optimization-plan.md

按顺序执行 Task 0-6，每个 Task 包含详细步骤和测试命令。
```

---

## 可选：直接开始某个 Task

**Task 0 - 添加依赖**：
```
请执行 docs/plans/2026-03-29-http-optimization-plan.md 的 Task 0：
在 Cargo.toml 添加 tokio-stream, local-ip-address, futures-util 依赖
```

**Task 1 - HTTP 保存文件**：
```
请执行 docs/plans/2026-03-29-http-optimization-plan.md 的 Task 1：
修改 http_server.rs，实现文件保存到 ~/Downloads/Thru/
```

**Task 2 - IP 认证**：
```
请执行 docs/plans/2026-03-29-http-optimization-plan.md 的 Task 2：
添加 IP 网段白名单认证
```

**Task 3 - 进度条**：
```
请执行 docs/plans/2026-03-29-http-optimization-plan.md 的 Task 3：
使用 Channel 模式重构 http_client.rs
```

**Task 4 - 设备 IP**：
```
请执行 docs/plans/2026-03-29-http-optimization-plan.md 的 Task 4：
使用 local_ip_address 自动检测本机 IP
```