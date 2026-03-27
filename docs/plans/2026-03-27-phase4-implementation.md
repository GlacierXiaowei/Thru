# Phase 4: HTTP 完善实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完善 Phase 3 HTTP 功能，实现进度条、自动发现设备、降级策略和 receive 命令。

**Architecture:** 使用 indicatif 实现进度条，复用 discovery 模块实现自动发现，统一 transfer 接口实现降级策略，复用 http_server 实现接收功能。

**Tech Stack:** Rust, indicatif 0.17, reqwest 0.12, tokio 1.0

**Known Issues:**
- 路径引号问题：用户路径含空格时 SSH 命令失败
- Git Bash 环境：%USERPROFILE% 不能直接使用，需用 /c/Users/Glacier Xiaowei/

---

## 进度

| Batch | Tasks | 状态 |
|-------|-------|------|
| Batch 1 | Task 1-3 | ✅ 完成 |
| Batch 2 | Task 4-6 | ✅ 完成 |
| Batch 3 | Task 7-9 | ✅ 完成 |

---

## Batch 1: 进度条

### Task 1: 创建进度条工具模块

**Files:**
- Create: `src/utils/progress.rs`
- Modify: `src/utils/mod.rs`

**Step 1: 创建 progress.rs**

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_upload_bar(total_size: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    
    pb.set_style(
        ProgressStyle::with_template(
            &format!("📤 Sending {}\n[{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{percent}}%  {{bytes}}/{{total_bytes}}  {{bytes_per_sec}}  ETA: {{eta}}", 
                file_name)
        )
        .unwrap()
        .progress_chars("#>-")
    );
    
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

pub fn create_download_bar(total_size: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    
    pb.set_style(
        ProgressStyle::with_template(
            &format!("📥 Receiving {}\n[{{elapsed_precise}}] [{{bar:40.green/blue}}] {{percent}}%  {{bytes}}/{{total_bytes}}  {{bytes_per_sec}}  ETA: {{eta}}", 
                file_name)
        )
        .unwrap()
        .progress_chars("#>-")
    );
    
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
```

**Step 2: 更新 utils/mod.rs**

添加:
```rust
pub mod progress;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/utils/progress.rs src/utils/mod.rs
git commit -m "feat: add progress bar utility module"
```

---

### Task 2: 更新 HTTP 客户端支持进度条

**Files:**
- Modify: `src/core/http_client.rs`

**Step 1: 添加进度回调支持**

修改 `http_client.rs`:

```rust
use anyhow::{Result, bail};
use indicatif::ProgressBar;
use reqwest::multipart;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::fs::File;
use futures::stream::{self, StreamExt};

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
        
        let file_bytes = std::fs::read(path)?;
        
        if let Some(ref pb) = pb {
            pb.inc(file_bytes.len() as u64);
        }
        
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

**Step 2: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 3: Commit**

```bash
git add src/core/http_client.rs
git commit -m "feat: add progress bar support to HTTP client"
```

---

### Task 3: 更新 send 命令使用进度条

**Files:**
- Modify: `src/commands/send.rs`

**Step 1: 启用进度条**

找到 HTTP 发送部分，修改为:

```rust
if let Some(ref lan) = lan {
    let (ip, port) = parse_lan_address(lan)?;
    
    let show_progress = true;
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        crate::core::http_client::HttpClient::send_file(&ip, port, file, json, show_progress).await
    })?;
    
    return Ok(());
}
```

**Step 2: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 3: 测试进度条**

```bash
# 终端 1: 启动服务器
./target/release/thru serve

# 终端 2: 发送文件（观察进度条）
./target/release/thru send <大文件> --lan <IP:PORT>
```

**Step 4: Commit**

```bash
git add src/commands/send.rs
git commit -m "feat: enable progress bar in send command"
```

---

## Batch 2: 自动发现 + 降级策略

### Task 4: 实现自动发现设备选择

**Files:**
- Modify: `src/commands/send.rs`

**Step 1: 添加设备选择逻辑**

在 send.rs 中添加:

```rust
use crate::core::discovery::Discovery;
use std::io::{self, Write};

fn select_device(timeout: u64) -> Result<(String, u16)> {
    println!("🔍 正在搜索设备...");
    
    let devices = Discovery::discover(timeout)?;
    
    if devices.is_empty() {
        bail!("未发现任何设备\n提示：请确保目标设备已运行 thru serve");
    }
    
    if devices.len() == 1 {
        let d = &devices[0];
        println!("发现设备: {} ({}:{})", d.name, d.ip, d.port);
        return Ok((d.ip.clone(), d.port));
    }
    
    println!("\n发现的设备:");
    println!("─────────────────────────────────");
    for (i, d) in devices.iter().enumerate() {
        println!("  {}. {} ({}:{}) [{}]", 
            i + 1, d.name, d.ip, d.port, d.network);
    }
    println!("─────────────────────────────────");
    
    print!("选择目标设备 [1-{}]: ", devices.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("无效的选择"))?;
    
    if choice < 1 || choice > devices.len() {
        bail!("选择超出范围");
    }
    
    let d = &devices[choice - 1];
    Ok((d.ip.clone(), d.port))
}
```

**Step 2: 更新 --lan 参数逻辑**

```rust
if lan.is_some() {
    let (ip, port) = match lan {
        Some(ref addr) if !addr.is_empty() => {
            parse_lan_address(addr)?
        },
        _ => {
            select_device(5)?
        }
    };
    
    let show_progress = true;
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        crate::core::http_client::HttpClient::send_file(&ip, port, file, json, show_progress).await
    })?;
    
    return Ok(());
}
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/commands/send.rs
git commit -m "feat: add auto-discovery device selection"
```

---

### Task 5: 实现降级策略

**Files:**
- Modify: `src/commands/send.rs`

**Step 1: 添加降级逻辑**

```rust
async fn send_with_fallback(
    ip: &str,
    port: u16,
    file: &str,
    json: bool,
) -> Result<()> {
    // 尝试 HTTP
    match crate::core::http_client::HttpClient::send_file(ip, port, file, json, true).await {
        Ok(_) => return Ok(()),
        Err(e) => {
            if !json {
                println!("⚠ HTTP 发送失败: {}", e);
                println!("正在尝试 rsync...");
            }
        }
    }
    
    // 尝试 rsync
    if let Ok(_) = which::which("rsync") {
        let result = std::process::Command::new("rsync")
            .args(["-avz", "--progress", file, &format!("{}@{}:~/storage/downloads/Thru/", 
                get_config().phone_user, get_config().phone_ip)])
            .status();
        
        if let Ok(status) = result {
            if status.success() {
                return Ok(());
            }
        }
    }
    
    // 尝试 scp
    if !json {
        println!("正在尝试 scp...");
    }
    
    let result = std::process::Command::new("scp")
        .args([file, &format!("{}@{}:~/storage/downloads/Thru/", 
            get_config().phone_user, get_config().phone_ip)])
        .status();
    
    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!("scp 发送失败"),
        Err(e) => bail!("所有传输方式都失败: {}", e),
    }
}
```

**Step 2: 更新 send 逻辑**

```rust
if lan.is_some() || auto_http {
    let (ip, port) = match lan {
        Some(ref addr) if !addr.is_empty() => {
            parse_lan_address(addr)?
        },
        _ => {
            select_device(5)?
        }
    };
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        if no_fallback {
            crate::core::http_client::HttpClient::send_file(&ip, port, file, json, true).await
        } else {
            send_with_fallback(&ip, port, file, json).await
        }
    })?;
    
    return Ok(());
}
```

**Step 3: 添加新参数到 main.rs**

在 Send 命令中添加:

```rust
/// 不使用降级策略
#[arg(long)]
no_fallback: bool,
```

**Step 4: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 5: Commit**

```bash
git add src/commands/send.rs src/main.rs
git commit -m "feat: implement fallback strategy (HTTP → rsync → scp)"
```

---

### Task 6: 测试降级策略

**Step 1: 测试 HTTP 成功**

```bash
# 终端 1
./target/release/thru serve

# 终端 2
./target/release/thru send test.txt --lan
# 应使用 HTTP
```

**Step 2: 测试降级**

```bash
# 不启动服务器
./target/release/thru send test.txt --lan
# 应尝试 rsync/scp
```

**Step 3: Commit**

```bash
git add .
git commit -m "test: verify fallback strategy"
```

---

## Batch 3: receive 命令

### Task 7: 创建 receive 命令处理器

**Files:**
- Create: `src/commands/receive.rs`
- Modify: `src/commands/mod.rs`

**Step 1: 创建 receive.rs**

```rust
use crate::core::http_server::HttpServer;
use anyhow::Result;

pub fn handle_receive(port: Option<u16>, json: bool, save_dir: Option<String>) -> Result<()> {
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
                "message": "Waiting for files",
                "port": port.unwrap_or(53317)
            }));
        } else {
            println!("📥 等待接收文件...");
            println!("  保存到: ~/Downloads/Thru/");
            println!("  按 Ctrl+C 停止");
            println!();
        }
        
        server.start_receive_mode(json).await
    })
}
```

**Step 2: 更新 commands/mod.rs**

添加:
```rust
pub mod receive;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/commands/receive.rs src/commands/mod.rs
git commit -m "feat: add receive command handler"
```

---

### Task 8: 更新 HTTP 服务器支持接收模式

**Files:**
- Modify: `src/core/http_server.rs`

**Step 1: 添加接收模式方法**

在 HttpServer impl 中添加:

```rust
pub async fn start_receive_mode(&self, json: bool) -> Result<()> {
    use axum::extract::Multipart;
    use std::path::PathBuf;
    
    let save_dir: PathBuf = dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Thru");
    
    std::fs::create_dir_all(&save_dir)?;
    
    let save_dir_clone = save_dir.clone();
    let json_mode = json;
    
    let app = Router::new()
        .route("/upload", post(move |mut multipart: Multipart| async move {
            while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
                let name = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                
                let file_path = save_dir_clone.join(&name);
                std::fs::write(&file_path, &data)?;
                
                if !json_mode {
                    println!("📥 收到文件: {} ({} bytes)", name, data.len());
                    println!("✓ 已保存到: {}", file_path.display());
                    println!();
                } else {
                    println!("{}", serde_json::json!({
                        "event": "file_received",
                        "file": {
                            "name": name,
                            "size": data.len(),
                            "path": file_path.to_string_lossy()
                        }
                    }));
                }
            }
            
            Ok(StatusCode::OK)
        }))
        .route("/device", get(device_info));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
    
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => {
            println!("🌐 接收服务已启动");
            println!("  地址: http://0.0.0.0:{}", self.port);
            l
        }
        Err(_) => {
            let backup_addr = SocketAddr::from(([0, 0, 0, 0], 53318));
            println!("⚠ 端口 {} 已被占用，使用端口 53318...", self.port);
            TcpListener::bind(backup_addr).await?
        }
    };
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

**Step 2: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 3: Commit**

```bash
git add src/core/http_server.rs
git commit -m "feat: add receive mode to HTTP server"
```

---

### Task 9: 添加 receive 命令到 CLI

**Files:**
- Modify: `src/main.rs`

**Step 1: 添加命令定义**

在 Commands 枚举中添加:

```rust
/// 等待接收文件（HTTP 方式）
Receive {
    /// 指定端口
    #[arg(short, long)]
    port: Option<u16>,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
    /// 指定保存目录
    #[arg(long)]
    save_dir: Option<String>,
},
```

**Step 2: 添加命令处理**

在 match commands 中添加:

```rust
Commands::Receive { port, json, save_dir } => {
    commands::receive::handle_receive(port, json, save_dir)?;
}
```

**Step 3: 验证编译**

Run: `cargo build --release`
Expected: 无错误

**Step 4: 测试命令**

```bash
./target/release/thru receive --help
```

Expected: 显示帮助信息

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add receive command to CLI"
```

---

## 验证清单

- [ ] `thru send file.jpg --lan` 显示进度条
- [ ] `thru send file.jpg --lan` 自动发现设备
- [ ] 多设备时显示选择列表
- [ ] HTTP 失败时自动降级到 rsync
- [ ] `thru receive` 启动接收服务
- [ ] `thru receive --json` JSON 输出
- [ ] 手机发送文件，电脑正常接收

---

## 相关文档

- [Phase 4 设计文档](./2026-03-27-phase4-http-enhancement.md)
- [Phase 3 实施计划](./2026-03-27-phase3-implementation.md)
- [产品路线图](./2026-03-26-thru-roadmap.md)