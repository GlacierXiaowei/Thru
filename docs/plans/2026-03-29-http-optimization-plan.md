# HTTP 优化实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 HTTP 服务端保存文件、IP 认证、进度条架构、设备 IP 硬编码四个问题。

**Architecture:** 
- HTTP 服务端使用 tokio::fs 异步写入文件
- IP 认证使用网段白名单（192.168/10/172.16/100.64）
- 进度条使用 Channel 模式：应用层控制进度，HTTP 层只接收 stream
- 设备发现使用 local_ip_address 自动检测本机 IP

**Tech Stack:** Rust, Axum, tokio_stream, local_ip_address

---

## 前置任务：添加依赖

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加新依赖**

在 `[dependencies]` 部分添加：

```toml
tokio-stream = "0.1"
local-ip-address = "0.6"
```

**Step 2: 验证依赖**

Run: `cargo check`
Expected: 编译成功，无错误

---

## Task 1: HTTP 服务端保存文件 (P0-1)

**Files:**
- Modify: `src/core/http_server.rs:68-77`

**Step 1: 添加必要的 use 语句**

在文件顶部添加：

```rust
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use chrono::Local;
```

**Step 2: 创建保存目录的辅助函数**

在 `HttpServer` impl 块之前添加：

```rust
fn get_receive_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    Ok(home.join("Downloads").join("Thru"))
}

async fn ensure_receive_dir() -> Result<PathBuf> {
    let dir = get_receive_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).await?;
    }
    Ok(dir)
}
```

**Step 3: 重写 upload handler**

替换原有的 `upload` 函数：

```rust
async fn upload(mut multipart: Multipart) -> Result<StatusCode, (StatusCode, String)> {
    let receive_dir = ensure_receive_dir()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    while let Some(field) = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? 
    {
        let name = field.file_name()
            .unwrap_or("unknown")
            .to_string();
        
        let data = field.bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");
        let filename = format!("{}_{}", timestamp, safe_name);
        let file_path = receive_dir.join(&filename);
        
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        file.write_all(&data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        file.flush()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
        println!("📥 已保存文件: {} ({} bytes)", file_path.display(), data.len());
    }
    
    Ok(StatusCode::OK)
}
```

**Step 4: 更新 Router 的错误处理**

修改 `upload` 路由，使其返回正确的错误类型。在 `Router::new()` 中：

```rust
.route("/upload", post(upload))
```

需要修改 upload 返回类型兼容 axum。完整修改：

```rust
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Result;

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let receive_dir = match ensure_receive_dir().await {
        Ok(d) => d,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };
    
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap_or("unknown").to_string();
        
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
            }
        };
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");
        let filename = format!("{}_{}", timestamp, safe_name);
        let file_path = receive_dir.join(&filename);
        
        match fs::File::create(&file_path).await {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&data).await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                if let Err(e) = file.flush().await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
        
        println!("📥 已保存文件: {} ({} bytes)", file_path.display(), data.len());
    }
    
    (StatusCode::OK, "OK".to_string()).into_response()
}
```

**Step 5: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 6: 手动测试**

终端 1 启动服务：
```bash
cargo run -- serve
```

终端 2 上传测试：
```bash
curl -X POST -F "file=@test.txt" http://localhost:53317/upload
```

Expected: 文件保存到 `~/Downloads/Thru/`，终端显示保存路径

---

## Task 2: IP 网段认证 (P0-2)

**Files:**
- Modify: `src/core/http_server.rs`

**Step 1: 添加 IP 验证函数**

在 `get_receive_dir` 函数之后添加：

```rust
fn is_ip_allowed(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    
    let first: u8 = parts[0].parse().unwrap_or(0);
    let second: u8 = parts[1].parse().unwrap_or(0);
    
    match first {
        10 => true,
        192 => second == 168,
        172 => (16..=31).contains(&second),
        100 => (64..=127).contains(&second),
        127 => true,
        _ => false,
    }
}
```

**Step 2: 修改 upload 函数添加 IP 验证**

在 `upload` 函数开头添加 IP 验证：

```rust
use axum::extract::ConnectInfo;
use std::net::SocketAddr as AxumSocketAddr;

async fn upload(
    ConnectInfo(addr): ConnectInfo<AxumSocketAddr>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    
    if !is_ip_allowed(&client_ip) {
        println!("⛔ 拒绝来自 {} 的上传请求", client_ip);
        return (StatusCode::FORBIDDEN, format!("IP {} not allowed", client_ip)).into_response();
    }
    
    println!("✅ 允许来自 {} 的上传请求", client_ip);
    
}
```

**Step 3: 更新 Server 启动配置**

修改 `start` 方法，添加 `ConnectInfo` 层：

```rust
use tower::ServiceBuilder;
use axum::extract::DefaultBodyLimit;

pub async fn start(&self) -> Result<()> {
    let app = Router::new()
        .route("/", get(root))
        .route("/upload", post(upload))
        .route("/files", get(list_files))
        .route("/device", get(device_info))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(axum::extract::ConnectInfo::<AxumSocketAddr>::layer())
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
    
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => {
            println!("🌐 HTTP 服务已启动");
            println!("  地址: http://0.0.0.0:{}", self.port);
            l
        }
        Err(_) if self.port == DEFAULT_PORT => {
            println!("⚠ 端口 {} 已被占用，正在使用备用端口 {}...", DEFAULT_PORT, BACKUP_PORT_1);
            let backup_addr = SocketAddr::from(([0, 0, 0, 0], BACKUP_PORT_1));
            TcpListener::bind(backup_addr).await?
        }
        Err(_) if self.port == BACKUP_PORT_1 => {
            println!("⚠ 端口 {}/{} 均不可用，正在使用备用端口 {}...", DEFAULT_PORT, BACKUP_PORT_1, BACKUP_PORT_2);
            let backup_addr = SocketAddr::from(([0, 0, 0, 0], BACKUP_PORT_2));
            TcpListener::bind(backup_addr).await?
        }
        Err(e) => anyhow::bail!("无法绑定任何端口: {}", e),
    };
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<AxumSocketAddr>(),
    ).await?;
    
    Ok(())
}
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 手动测试 IP 验证**

终端 1 启动服务：
```bash
cargo run -- serve
```

终端 2 本机测试（127.0.0.1 应该被允许）：
```bash
curl -X POST -F "file=@test.txt" http://127.0.0.1:53317/upload
```

Expected: 返回 200 OK，文件保存成功

---

## Task 3: Channel 模式进度条 (P1-1)

**Files:**
- Modify: `src/core/http_client.rs`

**Step 1: 添加必要的 use 语句**

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures_util::StreamExt;
```

**Step 2: 重构 send_file 函数**

完整替换 `send_file` 函数：

```rust
use anyhow::{Result, bail};
use reqwest::multipart;
use reqwest::Body;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct HttpClient;

impl HttpClient {
    pub async fn send_file(
        ip: &str,
        port: u16,
        file_path: &str,
        json: bool,
        show_progress: bool,
    ) -> Result<()> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            bail!("文件不存在: {}", file_path);
        }
        
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let file_size = std::fs::metadata(path)?.len();
        
        if !json {
            println!("📤 正在发送 {}...", file_name);
        }
        
        let url = format!("http://{}:{}/upload", ip, port);
        
        let pb = if show_progress && !json {
            Some(crate::utils::progress::create_upload_bar(file_size, &file_name))
        } else {
            None
        };
        
        let (tx, rx) = mpsc::channel::<Vec<u8>>(64);
        let bytes_sent = Arc::new(AtomicU64::new(0));
        let bytes_sent_clone = bytes_sent.clone();
        let pb_clone = pb.clone();
        let file_path_owned = file_path.to_string();
        
        let file_task = tokio::spawn(async move {
            let mut file = File::open(&file_path_owned).await.ok()?;
            let mut buf = vec![0u8; 64 * 1024];
            
            loop {
                match file.read(&mut buf).await {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        let chunk = buf[..n].to_vec();
                        bytes_sent_clone.fetch_add(n as u64, Ordering::SeqCst);
                        
                        if let Some(ref pb) = pb_clone {
                            pb.set_position(bytes_sent_clone.load(Ordering::SeqCst));
                        }
                        
                        if tx.send(chunk).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            
            Some(())
        });
        
        let stream = ReceiverStream::new(rx);
        let body = Body::wrap_stream(stream.map(Ok::<_, std::io::Error>));
        
        let file_part = multipart::Part::stream_with_length(body, file_size)
            .file_name(file_name.clone())
            .mime_str("application/octet-stream")?;
        
        let form = multipart::Form::new()
            .part("file", file_part);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .multipart(form)
            .send()
            .await?;
        
        let _ = file_task.await;
        
        if let Some(pb) = pb {
            pb.finish();
        }
        
        if response.status().is_success() {
            if json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "method": "http",
                    "file": {
                        "name": file_name,
                        "size": file_size
                    }
                }));
            } else {
                println!("✓ 发送成功");
            }
            Ok(())
        } else {
            bail!("HTTP 发送失败: {}", response.status())
        }
    }
}
```

**Step 3: 更新 Cargo.toml 确保依赖**

确认 `tokio-stream` 和 `futures-util` 已添加：

```toml
tokio-stream = "0.1"
futures-util = "0.3"
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 手动测试进度条**

终端 1 启动服务：
```bash
cargo run -- serve
```

终端 2 发送文件：
```bash
cargo run -- send test.txt --lan
```

Expected: 显示进度条，进度实时更新

---

## Task 4: 自动检测设备 IP (P1-2)

**Files:**
- Modify: `src/core/discovery.rs:82`

**Step 1: 添加 local_ip_address 依赖**

在 `Cargo.toml` 中：
```toml
local-ip-address = "0.6"
```

**Step 2: 添加 IP 检测函数**

在 `discovery.rs` 中添加：

```rust
use local_ip_address::local_ip;

fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "0.0.0.0".to_string(),
    }
}
```

**Step 3: 修改 respond 函数**

将 `discovery.rs:82` 的硬编码 IP 替换：

```rust
pub fn respond(port: u16, device_id: String) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), MULTICAST_PORT))?;
    socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
    
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    
    let local_ip = get_local_ip();
    
    let response = DeviceInfo {
        msg_type: "THRU_RESPONSE".to_string(),
        name: hostname,
        ip: local_ip,
        port,
        device_id,
        network: "lan".to_string(),
    };
    
    println!("📡 设备发现服务已启动 (IP: {})", response.ip);
    
    let mut buf = [0u8; 4096];
    
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(msg) = serde_json::from_slice::<DiscoverMessage>(&buf[..len]) {
                    if msg.msg_type == "THRU_DISCOVER" {
                        let response_bytes = serde_json::to_vec(&response)?;
                        socket.send_to(&response_bytes, addr)?;
                    }
                }
            }
            Err(_) => continue,
        }
    }
}
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 手动测试 IP 检测**

终端 1 启动发现服务：
```bash
cargo run -- serve
```

终端 2 发现设备：
```bash
cargo run -- discover
```

Expected: 显示设备实际 IP（如 192.168.x.x），而不是 0.0.0.0

---

## Task 5: 更新文档

**Files:**
- Modify: `docs/plans/2026-03-26-thru-roadmap.md`
- Modify: `docs/handover-prompt.md`

**Step 1: 更新 roadmap.md 问题状态**

找到问题表格部分，更新状态：

```markdown
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
```

**Step 2: 更新 handover-prompt.md**

更新待修复问题表格：

```markdown
## 待修复问题（按优先级）

| 优先级 | 问题 | 位置 | 说明 | 状态 |
|--------|------|------|------|------|
| **P0** | HTTP 服务端不保存文件 | http_server.rs | 功能完全缺失 | ✅ 已修复 |
| **P0** | HTTP 上传无认证 | http_server.rs | IP 网段白名单 | ✅ 已修复 |
| **P1** | 进度条架构错误 | http_client.rs | Channel 模式 | ✅ 已修复 |
| **P1** | 设备 IP 硬编码 | discovery.rs | 自动检测 IP | ✅ 已修复 |
| **P2** | 同步 I/O 阻塞 | transfer.rs | 改用 tokio::process | 📅 延后 |
| **P2** | 无并发上传 | 全局 | 添加 TaskRunner | 📅 延后 |
```

**Step 3: 添加 PIN 配对机制到 Phase 5 文档**

在 roadmap.md 的 Phase 5 部分添加：

```markdown
### PIN 配对机制（Phase 5）

**设计目标**：手机和电脑输入同一 PIN 后配对为可信设备。

**流程**：
1. 电脑端生成 6 位 PIN 码并显示
2. 手机端输入 PIN
3. 验证成功后，设备加入信任列表
4. 后续传输无需再次验证

**适用场景**：
- 局域网首次配对
- 跨网络首次配对（Tailscale）

**安全考虑**：
- PIN 有效期 60 秒
- 配对成功后生成设备 Token
- 支持取消配对
```

**Step 4: 提交文档更新**

```bash
git add docs/plans/2026-03-26-thru-roadmap.md docs/handover-prompt.md
git commit -m "docs: 更新问题修复状态，添加 PIN 配对设计"
```

---

## Task 6: 最终验证

**Step 1: 完整编译测试**

Run: `cargo build --release`
Expected: 编译成功，无警告

**Step 2: 功能测试清单**

1. ✅ HTTP 服务端启动：`cargo run -- serve`
2. ✅ 文件上传并保存：`curl -X POST -F "file=@test.txt" http://127.0.0.1:53317/upload`
3. ✅ IP 认证生效：检查终端日志
4. ✅ 进度条显示：`cargo run -- send test.txt --lan`
5. ✅ 设备发现：`cargo run -- discover`

**Step 3: 提交所有更改**

```bash
git add .
git commit -m "feat: HTTP 优化 - 文件保存、IP认证、进度条、设备IP检测"
```

---

## 实现顺序建议

1. **Task 0**: 添加依赖
2. **Task 1**: HTTP 服务端保存文件（P0-1）
3. **Task 2**: IP 网段认证（P0-2）
4. **Task 3**: Channel 模式进度条（P1-1）
5. **Task 4**: 自动检测设备 IP（P1-2）
6. **Task 5**: 更新文档
7. **Task 6**: 最终验证

---

## 测试命令汇总

```bash
# 启动 HTTP 服务
cargo run -- serve

# 测试上传（另一个终端）
curl -X POST -F "file=@test.txt" http://127.0.0.1:53317/upload

# 测试发送（带进度条）
cargo run -- send test.txt --lan

# 测试设备发现
cargo run -- discover

# 检查编译
cargo check
cargo build --release
```

---

## 依赖变更

```toml
# 新增依赖
tokio-stream = "0.1"
local-ip-address = "0.6"
futures-util = "0.3"
```

---

## 文件变更摘要

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `Cargo.toml` | Modify | 添加 3 个新依赖 |
| `src/core/http_server.rs` | Modify | 文件保存 + IP 认证 |
| `src/core/http_client.rs` | Modify | Channel 模式进度条 |
| `src/core/discovery.rs` | Modify | 自动检测 IP |
| `docs/plans/2026-03-26-thru-roadmap.md` | Modify | 更新问题状态 |
| `docs/handover-prompt.md` | Modify | 更新问题状态 |