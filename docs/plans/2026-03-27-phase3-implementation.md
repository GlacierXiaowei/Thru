# Phase 3: HTTP 局域网传输实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 HTTP 局域网传输功能，支持 `thru serve`、`thru discover`、`thru send --lan` 命令。

**Architecture:** 使用 axum + tokio 构建 HTTP 服务器，UDP 组播实现设备发现，reqwest 作为 HTTP 客户端。端口自动降级：53317 → 53318 → 8080。

**Tech Stack:** Rust, axum 0.7, tokio 1.0, reqwest 0.12, tower-http 0.5, uuid 1.0

**Known Issues:**
- 路径引号问题：用户路径含空格时 SSH 命令失败，需要在相关文件中加引号
- 手机端使用 Python 临时方案：`python -m http.server 53317 --directory ~/storage/downloads/Thru`

---

## 进度

| Batch | Tasks | 状态 |
|-------|-------|------|
| Batch 1 | Task 1-4 | ✅ 完成 |
| Batch 2 | Task 5-7 | ✅ 完成 |
| Batch 3 | Task 8-10 | ✅ 完成 |

---

## Batch 1: HTTP 服务器基础

### Task 1: 添加依赖

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加新依赖**

```toml
[dependencies]
# ... 现有依赖 ...

# HTTP 服务器
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# HTTP 客户端
reqwest = { version = "0.12", features = ["multipart", "stream"] }

# 设备标识
uuid = { version = "1.0", features = ["v4"] }
hostname = "0.4"
```

**Step 2: 验证依赖下载**

Run: `cargo check`
Expected: 无错误

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add HTTP server dependencies for Phase 3"
```

---

### Task 2: 创建 HTTP 服务器模块

**Files:**
- Create: `src/core/http_server.rs`
- Modify: `src/core/mod.rs`

**Step 1: 创建 http_server.rs 基础结构**

```rust
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Result;

const DEFAULT_PORT: u16 = 53317;
const BACKUP_PORT_1: u16 = 53318;
const BACKUP_PORT_2: u16 = 8080;

pub struct HttpServer {
    port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub device_id: String,
    pub port: u16,
}

impl HttpServer {
    pub fn new() -> Self {
        Self { port: DEFAULT_PORT }
    }

    pub fn with_port(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/upload", post(upload))
            .route("/files", get(list_files))
            .route("/device", get(device_info));

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

        let actual_port = listener.local_addr()?.port();
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Thru",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

async fn upload(mut multipart: Multipart) -> Result<StatusCode, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        println!("📥 收到文件: {} ({} bytes)", name, data.len());
    }
    
    Ok(StatusCode::OK)
}

async fn list_files() -> Json<serde_json::Value> {
    Json(json!({
        "files": [],
        "total": 0
    }))
}

async fn device_info() -> Json<serde_json::Value> {
    Json(json!({
        "name": hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default(),
        "device_id": uuid::Uuid::new_v4().to_string(),
        "port": DEFAULT_PORT
    }))
}
```

**Step 2: 更新 core/mod.rs**

```rust
pub mod http_server;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/core/http_server.rs src/core/mod.rs Cargo.toml Cargo.lock
git commit -m "feat: add HTTP server module with port fallback"
```

---

### Task 3: 实现 thru serve 命令

**Files:**
- Create: `src/commands/serve.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 创建 serve.rs**

```rust
use crate::core::http_server::HttpServer;
use anyhow::Result;

pub fn handle_serve(port: Option<u16>, json: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        let server = if let Some(p) = port {
            HttpServer::with_port(p)
        } else {
            HttpServer::new()
        };
        
        if json {
            println!("{}", serde_json::json!({
                "success": true,
                "message": "HTTP server starting",
                "port": port.unwrap_or(53317)
            }));
        }
        
        server.start().await
    })
}
```

**Step 2: 更新 commands/mod.rs**

```rust
pub mod serve;
```

**Step 3: 更新 main.rs 添加命令**

在 `Commands` 枚举中添加：

```rust
/// 启动 HTTP 文件服务
Serve {
    /// 指定端口
    #[arg(short, long)]
    port: Option<u16>,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},
```

在 `match commands` 中添加：

```rust
Commands::Serve { port, json } => {
    commands::serve::handle_serve(port, json)?;
}
```

**Step 4: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 5: 测试命令**

Run: `./target/release/thru serve --help`
Expected: 显示帮助信息

**Step 6: Commit**

```bash
git add src/commands/serve.rs src/commands/mod.rs src/main.rs
git commit -m "feat: implement thru serve command"
```

---

### Task 4: 测试 HTTP 服务器

**Step 1: 启动服务器**

Run: `./target/release/thru serve`
Expected: 显示 "HTTP 服务已启动"

**Step 2: 测试根路由**

Run (另一个终端): `curl http://localhost:53317/`
Expected:
```json
{"name":"Thru","status":"running","version":"0.1.0"}
```

**Step 3: 测试文件上传**

Run:
```bash
echo "test content" > test.txt
curl -X POST http://localhost:53317/upload -F "file=@test.txt"
rm test.txt
```
Expected: 收到文件提示

**Step 4: Commit 测试结果**

```bash
git add .
git commit -m "test: verify HTTP server functionality"
```

---

## Batch 2: 设备发现

### Task 5: 实现 UDP 组播发现模块

**Files:**
- Create: `src/core/discovery.rs`
- Modify: `src/core/mod.rs`

**Step 1: 创建 discovery.rs**

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 12, 34, 56);
const MULTICAST_PORT: u16 = 53317;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoverMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub device_id: String,
    pub network: String,
}

pub struct Discovery;

impl Discovery {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(timeout_secs: u64) -> Result<Vec<DeviceInfo>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
        
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        
        let discover_msg = DiscoverMessage {
            msg_type: "THRU_DISCOVER".to_string(),
            version: "1.0".to_string(),
        };
        let msg_bytes = serde_json::to_vec(&discover_msg)?;
        
        let dest = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_PORT);
        socket.send_to(&msg_bytes, dest)?;
        
        println!("🔍 正在搜索局域网设备...");
        
        let mut devices = Vec::new();
        let mut buf = [0u8; 4096];
        let start = std::time::Instant::now();
        
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            match socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    if let Ok(device) = serde_json::from_slice::<DeviceInfo>(&buf[..len]) {
                        if device.msg_type == "THRU_RESPONSE" {
                            devices.push(device);
                        }
                    }
                }
                Err(_) => break,
            }
        }
        
        Ok(devices)
    }

    pub fn respond(port: u16, device_id: String) -> Result<()> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), MULTICAST_PORT))?;
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        
        let response = DeviceInfo {
            msg_type: "THRU_RESPONSE".to_string(),
            name: hostname,
            ip: "0.0.0.0".to_string(),
            port,
            device_id,
            network: "lan".to_string(),
        };
        
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
}
```

**Step 2: 更新 core/mod.rs**

```rust
pub mod discovery;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/core/discovery.rs src/core/mod.rs
git commit -m "feat: add UDP multicast discovery module"
```

---

### Task 6: 实现 thru discover 命令

**Files:**
- Create: `src/commands/discover.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 创建 discover.rs**

```rust
use crate::core::discovery::Discovery;
use anyhow::Result;

pub fn handle_discover(timeout: u64, json: bool) -> Result<()> {
    let devices = Discovery::discover(timeout)?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "devices": devices,
            "total": devices.len()
        }))?);
    } else {
        if devices.is_empty() {
            println!("未发现任何设备");
            println!("\n提示：请确保目标设备已运行 thru serve 或 Python HTTP 服务");
        } else {
            println!("\n发现的设备：");
            println!("─────────────────────────────────");
            for (i, d) in devices.iter().enumerate() {
                println!("  {}. {} ({}:{}) [{}]", 
                    i + 1, d.name, d.ip, d.port, d.network);
            }
            println!("─────────────────────────────────");
            println!("共发现 {} 台设备", devices.len());
        }
    }
    
    Ok(())
}
```

**Step 2: 更新 commands/mod.rs**

```rust
pub mod discover;
```

**Step 3: 更新 main.rs 添加命令**

在 `Commands` 枚举中添加：

```rust
/// 发现局域网设备
Discover {
    /// 搜索超时（秒）
    #[arg(short, long, default_value = "5")]
    timeout: u64,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},
```

在 `match commands` 中添加：

```rust
Commands::Discover { timeout, json } => {
    commands::discover::handle_discover(timeout, json)?;
}
```

**Step 4: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 5: Commit**

```bash
git add src/commands/discover.rs src/commands/mod.rs src/main.rs
git commit -m "feat: implement thru discover command"
```

---

### Task 7: 集成设备发现到 serve 命令

**Files:**
- Modify: `src/commands/serve.rs`
- Modify: `src/core/http_server.rs`

**Step 1: 更新 http_server.rs 支持发现响应**

在 `HttpServer::start` 方法中添加发现响应线程。

**Step 2: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 3: Commit**

```bash
git add src/commands/serve.rs src/core/http_server.rs
git commit -m "feat: integrate discovery response into serve command"
```

---

## Batch 3: HTTP 发送集成

### Task 8: 实现 HTTP 客户端模块

**Files:**
- Create: `src/core/http_client.rs`
- Modify: `src/core/mod.rs`

**Step 1: 创建 http_client.rs**

```rust
use anyhow::{Result, bail};
use reqwest::multipart;
use std::path::Path;

pub struct HttpClient;

impl HttpClient {
    pub async fn send_file(
        ip: &str,
        port: u16,
        file_path: &str,
        json: bool,
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
        
        let file_bytes = std::fs::read(path)?;
        
        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(file_name.clone());
        
        let form = multipart::Form::new()
            .part("file", file_part);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .multipart(form)
            .send()
            .await?;
        
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

**Step 2: 更新 core/mod.rs**

```rust
pub mod http_client;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/core/http_client.rs src/core/mod.rs
git commit -m "feat: add HTTP client module for file sending"
```

---

### Task 9: 更新 send 命令支持 --lan

**Files:**
- Modify: `src/commands/send.rs`
- Modify: `src/main.rs`

**Step 1: 更新 send.rs 支持 --lan**

**Step 2: 更新 main.rs 命令参数**

添加 `--lan` 参数到 Send 命令。

**Step 3: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/commands/send.rs src/main.rs
git commit -m "feat: add --lan flag to send command for HTTP transfer"
```

---

### Task 10: 测试和文档更新

**Step 1: 测试完整流程**

```bash
# 终端 1: 启动服务器
./target/release/thru serve

# 终端 2: 发现设备
./target/release/thru discover

# 终端 3: 发送文件
echo "test" > test.txt
./target/release/thru send test.txt --lan
rm test.txt
```

**Step 2: 更新 README**

添加 Phase 3 新命令说明。

**Step 3: Commit**

```bash
git add .
git commit -m "docs: update README and mark Phase 3 complete"
```

---

## 验证清单

- [x] `thru serve` 启动 HTTP 服务
- [x] `thru serve --port 8080` 指定端口
- [x] 端口降级：53317 → 53318 → 8080
- [x] `thru discover` 发现设备
- [x] `thru discover --json` JSON 输出
- [x] `thru send file.txt --lan` HTTP 发送
- [x] `curl http://localhost:53317/` 返回服务信息
- [x] 文件上传功能正常

---

## 手机端临时测试

```bash
# 在手机 Termux 中运行
pkg install python
python -m http.server 53317 --directory ~/storage/downloads/Thru

# 电脑端发现
./target/release/thru discover
```

---

## 相关文档

- [Phase 3 设计文档](./2026-03-27-phase3-http-transfer.md)
- [产品路线图](./2026-03-26-thru-roadmap.md)
- [Phase 2 实施计划](./2026-03-26-phase2-implementation.md)