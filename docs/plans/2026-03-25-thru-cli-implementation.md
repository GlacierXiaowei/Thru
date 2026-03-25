# Thru CLI 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现完整的 Thru CLI，支持手机-电脑文件互传

**架构:** 单 crate 结构，内部按 commands/core/utils 分层，Phase 2 前可拆分为 thru-lib + thru-cli

**Tech Stack:** Rust 1.75+, clap 4.5, serde, toml, notify, chrono, dirs

---

## Phase 1.1: 项目初始化

### Task 1: 创建 Rust 项目

**文件:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/commands/mod.rs`
- Create: `src/core/mod.rs`
- Create: `src/utils/mod.rs`

**Step 1: 创建 Cargo.toml**

```toml
[package]
name = "thru"
version = "0.1.0"
edition = "2021"
description = "手机-电脑文件互传工具"

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

**Step 2: 创建 main.rs 骨架**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "thru", version, about = "手机-电脑文件互传工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Status,
    Start,
    Stop,
    Send { file: String },
    Receive,
    List,
    History,
    Config,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => println!("Status command"),
        Some(Commands::Start) => println!("Start command"),
        Some(Commands::Stop) => println!("Stop command"),
        Some(Commands::Send { file }) => println!("Send: {}", file),
        Some(Commands::Receive) => println!("Receive command"),
        Some(Commands::List) => println!("List command"),
        Some(Commands::History) => println!("History command"),
        Some(Commands::Config) => println!("Config command"),
        None => println!("使用 --help 查看帮助"),
    }
}
```

**Step 3: 创建模块文件**

`src/lib.rs`:
```rust
pub mod commands;
pub mod core;
pub mod utils;
```

`src/commands/mod.rs`:
```rust
// pub mod status;
// pub mod start;
// pub mod stop;
// pub mod send;
// pub mod receive;
// pub mod list;
// pub mod config;
// pub mod history;
```

`src/core/mod.rs`:
```rust
// pub mod ssh_manager;
// pub mod tailscale;
// pub mod transfer;
// pub mod file_watcher;
// pub mod config;
// pub mod history;
```

`src/utils/mod.rs`:
```rust
// pub mod output;
```

**Step 4: 验证项目可编译**

Run: `cargo build`
Expected: 编译成功

**Step 5: 提交**

```bash
git add .
git commit -m "feat: 初始化 Rust 项目骨架"
```

---

## Phase 1.2: Config 模块

### Task 2: 实现配置管理

**文件:**
- Create: `src/core/config.rs`
- Modify: `src/core/mod.rs`

**Step 1: 定义配置结构体**

`src/core/config.rs`:
```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub aliases: std::collections::HashMap<String, String>,
    pub paths: PathsConfig,
    pub ssh: SshConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub phone_ip: String,
    pub phone_user: String,
    pub phone_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathsConfig {
    pub receive_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConfig {
    pub auto_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device: DeviceConfig {
                phone_ip: String::new(),
                phone_user: String::new(),
                phone_port: 8022,
            },
            aliases: std::collections::HashMap::new(),
            paths: PathsConfig {
                receive_dir: "~/Downloads/Thru".to_string(),
            },
            ssh: SshConfig {
                auto_start: true,
            },
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".thru")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
```

**Step 2: 更新 mod.rs**

`src/core/mod.rs`:
```rust
pub mod config;

// pub mod ssh_manager;
// pub mod tailscale;
// pub mod transfer;
// pub mod file_watcher;
// pub mod history;
```

**Step 3: 验证编译**

Run: `cargo build`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/core/config.rs src/core/mod.rs
git commit -m "feat: 实现配置管理模块"
```

---

## Phase 1.3: Config 命令

### Task 3: 实现 config 子命令

**文件:**
- Create: `src/commands/config.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 config 命令**

`src/commands/config.rs`:
```rust
use crate::core::config::{load_config, save_config, Config};
use anyhow::Result;

pub fn handle_show() -> Result<()> {
    let config = load_config()?;
    println!("📱 设备配置:");
    println!("  IP: {}", config.device.phone_ip);
    println!("  用户: {}", config.device.phone_user);
    println!("  端口: {}", config.device.phone_port);
    println!("\n📁 路径:");
    println!("  接收目录: {}", config.paths.receive_dir);
    println!("\n🔧 SSH:");
    println!("  自动启动: {}", config.ssh.auto_start);
    if !config.aliases.is_empty() {
        println!("\n🏷️  设备别名:");
        for (ip, alias) in &config.aliases {
            println!("  {} → {}", ip, alias);
        }
    }
    Ok(())
}

pub fn handle_set_ip(ip: &str) -> Result<()> {
    let mut config = load_config()?;
    config.device.phone_ip = ip.to_string();
    save_config(&config)?;
    println!("✓ 手机 IP 已设置为: {}", ip);
    Ok(())
}

pub fn handle_get_ip() -> Result<()> {
    let config = load_config()?;
    println!("{}", config.device.phone_ip);
    Ok(())
}

pub fn handle_set_alias(ip: &str, alias: &str) -> Result<()> {
    let mut config = load_config()?;
    config.aliases.insert(ip.to_string(), alias.to_string());
    save_config(&config)?;
    println!("✓ 设备别名已设置: {} → {}", ip, alias);
    Ok(())
}
```

**Step 2: 更新 mod.rs**

`src/commands/mod.rs`:
```rust
pub mod config;

// pub mod status;
// pub mod start;
// pub mod stop;
// pub mod send;
// pub mod receive;
// pub mod list;
// pub mod history;
```

**Step 3: 更新 main.rs 集成 config 命令**

```rust
use clap::Parser;
use anyhow::Result;

mod core;
mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "thru", version, about = "手机-电脑文件互传工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Status,
    Start,
    Stop,
    Send { file: String },
    Receive,
    List,
    History,
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    Show,
    SetIp { ip: String },
    GetIp,
    SetAlias { ip: String, name: String },
    AutoDetect,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => commands::config::handle_show()?,
            ConfigAction::SetIp { ip } => commands::config::handle_set_ip(&ip)?,
            ConfigAction::GetIp => commands::config::handle_get_ip()?,
            ConfigAction::SetAlias { ip, name } => commands::config::handle_set_alias(&ip, &name)?,
            ConfigAction::AutoDetect => println!("Auto-detect 未实现"),
        },
        Some(Commands::Status) => println!("Status command"),
        Some(Commands::Start) => println!("Start command"),
        Some(Commands::Stop) => println!("Stop command"),
        Some(Commands::Send { file }) => println!("Send: {}", file),
        Some(Commands::Receive) => println!("Receive command"),
        Some(Commands::List) => println!("List command"),
        Some(Commands::History) => println!("History command"),
        None => println!("使用 --help 查看帮助"),
    }
    Ok(())
}
```

**Step 4: 验证编译并测试**

Run: `cargo build`
Run: `cargo run -- config show`
Run: `cargo run -- config set-ip 100.118.195.54`
Expected: 正常输出

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 实现 config 子命令"
```

---

## Phase 1.4: Status 命令

### Task 4: 实现 SSH 状态检测

**文件:**
- Create: `src/core/ssh_manager.rs`
- Create: `src/commands/status.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 ssh_manager**

`src/core/ssh_manager.rs`:
```rust
use std::process::Command;
use anyhow::Result;

#[derive(Debug)]
pub struct SshStatus {
    pub running: bool,
    pub port: u16,
}

pub fn check_ssh_server() -> Result<SshStatus> {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Service sshd | Select-Object -ExpandProperty Status"])
        .output()?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    let running = status == "running";

    Ok(SshStatus {
        running,
        port: 22,
    })
}
```

**Step 2: 实现 status 命令**

`src/commands/status.rs`:
```rust
use crate::core::ssh_manager;
use anyhow::Result;

pub fn handle_status() -> Result<()> {
    let ssh = ssh_manager::check_ssh_server()?;

    let ssh_icon = if ssh.running { "●" } else { "○" };
    let ssh_text = if ssh.running { "Running" } else { "Stopped" };

    println!("SSH Server:  {} {}", ssh_icon, ssh_text);
    println!("Tailscale:   ○ Not implemented");
    println!("Phone IP:    (run config show)");
    Ok(())
}
```

**Step 3: 更新 mod.rs 文件**

`src/core/mod.rs`:
```rust
pub mod config;
pub mod ssh_manager;

// pub mod tailscale;
// pub mod transfer;
// pub mod file_watcher;
// pub mod history;
```

`src/commands/mod.rs`:
```rust
pub mod config;
pub mod status;

// pub mod start;
// pub mod stop;
// pub mod send;
// pub mod receive;
// pub mod list;
// pub mod history;
```

**Step 4: 更新 main.rs**

```rust
// 在 match 中添加
Some(Commands::Status) => commands::status::handle_status()?,
```

**Step 5: 验证**

Run: `cargo run -- status`
Expected: 显示 SSH 服务状态

**Step 6: 提交**

```bash
git add src/
git commit -m "feat: 实现 status 命令（SSH 状态检测）"
```

---

## Phase 1.5: Start/Stop 命令

### Task 5: 实现 SSH Server 控制

**文件:**
- Create: `src/commands/start.rs`
- Create: `src/commands/stop.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`
- Modify: `src/core/ssh_manager.rs`

**Step 1: 添加控制函数到 ssh_manager**

`src/core/ssh_manager.rs` 追加:
```rust
pub fn start_ssh_server() -> Result<()> {
    Command::new("powershell")
        .args(["-Command", "Start-Service sshd"])
        .output()?;
    println!("✓ SSH Server 已启动");
    Ok(())
}

pub fn stop_ssh_server() -> Result<()> {
    Command::new("powershell")
        .args(["-Command", "Stop-Service sshd"])
        .output()?;
    println!("✓ SSH Server 已停止");
    Ok(())
}
```

**Step 2: 实现 start 命令**

`src/commands/start.rs`:
```rust
use crate::core::ssh_manager;
use anyhow::Result;

pub fn handle_start() -> Result<()> {
    ssh_manager::start_ssh_server()
}
```

**Step 3: 实现 stop 命令**

`src/commands/stop.rs`:
```rust
use crate::core::ssh_manager;
use anyhow::Result;

pub fn handle_stop() -> Result<()> {
    ssh_manager::stop_ssh_server()
}
```

**Step 4: 更新 mod.rs**

`src/commands/mod.rs`:
```rust
pub mod config;
pub mod status;
pub mod start;
pub mod stop;

// pub mod send;
// pub mod receive;
// pub mod list;
// pub mod history;
```

**Step 5: 更新 main.rs**

```rust
Some(Commands::Start) => commands::start::handle_start()?,
Some(Commands::Stop) => commands::stop::handle_stop()?,
```

**Step 6: 验证**

Run: `cargo run -- status`
Run: `cargo run -- start`
Run: `cargo run -- status`
Run: `cargo run -- stop`
Run: `cargo run -- status`
Expected: 状态变化

**Step 7: 提交**

```bash
git add src/
git commit -m "feat: 实现 start/stop 命令"
```

---

## Phase 1.6: Transfer 模块 + Send 命令

### Task 6: 实现文件传输

**文件:**
- Create: `src/core/transfer.rs`
- Create: `src/commands/send.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 transfer 模块**

`src/core/transfer.rs`:
```rust
use std::process::Command;
use crate::core::config::Config;
use anyhow::Result;

pub fn send_file(config: &Config, file_path: &str, recursive: bool) -> Result<()> {
    let dest = format!(
        "{}@{}:{}",
        config.device.phone_user,
        config.device.phone_ip,
        "~/storage/downloads/"
    );

    let mut args = vec![
        "-P".to_string(),
        config.device.phone_port.to_string(),
    ];

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest);

    println!("📤 正在发送 {}...", file_path);

    let output = Command::new("scp")
        .args(&args)
        .output()?;

    if output.status.success() {
        println!("✓ 发送成功");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("✗ 发送失败: {}", stderr);
    }

    Ok(())
}
```

**Step 2: 实现 send 命令**

`src/commands/send.rs`:
```rust
use crate::core::config::load_config;
use crate::core::transfer;
use anyhow::Result;

pub fn handle_send(file: &str, recursive: bool) -> Result<()> {
    let config = load_config()?;
    transfer::send_file(&config, file, recursive)
}
```

**Step 3: 更新 main.rs**

```rust
#[derive(clap::Subcommand)]
enum Commands {
    // ...
    Send {
        file: String,
        /// 递归发送文件夹
        #[arg(short, long)]
        recursive: bool,
    },
    // ...
}

// 在 match 中
Some(Commands::Send { file, recursive }) => commands::send::handle_send(&file, recursive)?,
```

**Step 4: 验证**

Run: `cargo run -- config show`
Run: `cargo run -- send --help`
Expected: 显示 send 命令帮助

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 实现 send 命令（SCP 文件传输）"
```

---

## Phase 1.7: List 命令

### Task 7: 实现文件列表

**文件:**
- Create: `src/commands/list.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 list 命令**

`src/commands/list.rs`:
```rust
use crate::core::config::load_config;
use std::fs;
use anyhow::Result;
use std::path::PathBuf;

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = dirs::home_dir().unwrap();
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

pub fn handle_list(show_hidden: bool) -> Result<()> {
    let config = load_config()?;
    let dir = expand_tilde(&config.paths.receive_dir);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        println!("📁 已创建接收目录: {:?}", dir);
        return Ok(());
    }

    println!("📁 {:?} 内容:", dir);
    println!("");

    let entries = fs::read_dir(&dir)?;
    let mut count = 0;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        let size = metadata.len();
        let size_str = format_size(size);
        let is_dir = metadata.is_dir();

        let icon = if is_dir { "📁" } else { "  " };
        println!("{}{} ({})", icon, name, size_str);
        count += 1;
    }

    if count == 0 {
        println!("  (空)");
    } else {
        println!("\n  共 {} 项", count);
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
```

**Step 2: 更新 main.rs**

```rust
List {
    /// 显示隐藏文件
    #[arg(short, long)]
    all: bool,
},

// 在 match 中
Some(Commands::List { all }) => commands::list::handle_list(all)?,
```

**Step 3: 验证**

Run: `cargo run -- list`
Run: `cargo run -- list -a`
Expected: 显示接收目录内容

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: 实现 list 命令"
```

---

## Phase 1.8: File Watcher + Receive 命令

### Task 8: 实现文件监控

**文件:**
- Create: `src/core/file_watcher.rs`
- Create: `src/commands/receive.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 file_watcher**

`src/core/file_watcher.rs`:
```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::mpsc::channel;
use std::path::Path;
use std::time::Duration;
use anyhow::Result;

pub fn watch_directory(path: &Path, callback: impl Fn(&str)) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            for path in &event.paths {
                if let EventKind::Create(_) = event.kind {
                    if let Some(name) = path.file_name() {
                        tx.send(name.to_string_lossy().to_string()).ok();
                    }
                }
            }
        }
    })?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    println!("👁️  监控中: {:?}", path);
    println!("   按 Ctrl+C 停止\n");

    for filename in rx {
        callback(&filename);
    }

    Ok(())
}
```

**Step 2: 实现 receive 命令**

`src/commands/receive.rs`:
```rust
use crate::core::config::load_config;
use crate::core::file_watcher;
use chrono::Local;
use std::path::PathBuf;
use anyhow::Result;

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = dirs::home_dir().unwrap();
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

pub fn handle_receive(watch: bool) -> Result<()> {
    let config = load_config()?;
    let dir = expand_tilde(&config.paths.receive_dir);

    if !std::fs::exists(&dir)? {
        std::fs::create_dir_all(&dir)?;
    }

    if watch {
        println!("📥 接收监控启动");
        println!("");
        file_watcher::watch_directory(&dir, |filename| {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("[{}] 📥 收到文件", now);
            println!("  文件: {}", filename);
            println!("  设备: (待实现 Tailscale 识别)");
            println!("");
        })?;
    } else {
        // 简单列出当前文件
        println!("📁 已接收文件:");
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            println!("  - {}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}
```

**Step 3: 更新 main.rs**

```rust
Receive {
    /// 实时监控新文件
    #[arg(long)]
    watch: bool,
},

// 在 match 中
Some(Commands::Receive { watch }) => commands::receive::handle_receive(watch)?,
```

**Step 4: 验证**

Run: `cargo run -- receive`
Run: `cargo run -- receive --watch`
然后在另一个终端发一个文件，观察输出
Expected: 显示新文件

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 实现 receive 命令（文件监控）"
```

---

## Phase 1.9: History 模块

### Task 9: 实现历史记录

**文件:**
- Create: `src/core/history.rs`
- Create: `src/commands/history.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 实现 history 模块**

`src/core/history.rs`:
```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub entry_type: String,
    pub timestamp: DateTime<Local>,
    pub device: DeviceInfo,
    pub file: FileInfo,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub alias: Option<String>,
    pub ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
}

fn get_history_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".thru").join("history.json")
}

pub fn load_history() -> Result<Vec<HistoryEntry>> {
    let path = get_history_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_history(history: &[HistoryEntry]) -> Result<()> {
    let path = get_history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(history)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn add_entry(entry: HistoryEntry) -> Result<()> {
    let mut history = load_history()?;
    history.push(entry);
    save_history(&history)?;
    Ok(())
}

pub fn clear_history() -> Result<()> {
    save_history(&[])?;
    Ok(())
}

pub fn keep_history(n: usize) -> Result<()> {
    let history = load_history()?;
    let len = history.len();
    if len > n {
        let trimmed: Vec<HistoryEntry> = history.into_iter().skip(len - n).collect();
        save_history(&trimmed)?;
    }
    Ok(())
}
```

**Step 2: 实现 history 命令**

`src/commands/history.rs`:
```rust
use crate::core::history::{self, HistoryEntry};
use chrono::Local;
use anyhow::Result;

pub fn handle_history(show_all: bool, clear: bool, keep: Option<usize>) -> Result<()> {
    if clear {
        history::clear_history()?;
        println!("✓ 历史记录已清除");
        return Ok(());
    }

    if let Some(n) = keep {
        history::keep_history(n)?;
        println!("✓ 已保留最近 {} 条记录", n);
        return Ok(());
    }

    let entries = history::load_history()?;

    if entries.is_empty() {
        println!("暂无传输记录");
        return Ok(());
    }

    let display_count = if show_all { entries.len() } else { std::cmp::min(50, entries.len()) };
    let start = entries.len() - display_count;

    println!("📋 传输历史 (最近 {} 条):\n", display_count);

    for entry in &entries[start..] {
        let time = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let icon = if entry.entry_type == "send" { "📤" } else { "📥" };
        println!("[{}] {} {}", time, icon, entry.file.name);
        println!("  设备: {}", entry.device.name);
        println!("  大小: {}", format_size(entry.file.size));
        println!("");
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
```

**Step 3: 更新 main.rs**

```rust
History {
    /// 显示全部记录
    #[arg(long)]
    all: bool,
    /// 清除所有历史
    #[arg(long)]
    clear: bool,
    /// 只保留最近 n 条
    #[arg(long)]
    keep: Option<usize>,
},

// 在 match 中
Some(Commands::History { all, clear, keep }) => commands::history::handle_history(all, clear, keep)?,
```

**Step 4: 验证**

Run: `cargo run -- history`
Run: `cargo run -- history --clear`
Expected: 正常输出

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 实现 history 命令"
```

---

## Phase 1.10: Tailscale 检测

### Task 10: 实现 Tailscale 设备识别

**文件:**
- Create: `src/core/tailscale.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/commands/status.rs`
- Modify: `src/commands/receive.rs`

**Step 1: 实现 tailscale 模块**

`src/core/tailscale.rs`:
```rust
use std::process::Command;
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct TailscaleStatus {
    #[serde(rename = "Self")]
    pub self_device: TailscaleDevice,
    #[serde(rename = "Peer")]
    pub peers: std::collections::HashMap<String, TailscaleDevice>,
}

#[derive(Debug, Deserialize)]
pub struct TailscaleDevice {
    #[serde(rename = "DNSName")]
    pub dns_name: String,
    #[serde(rename = "TailscaleIPs")]
    pub tailscale_ips: Vec<String>,
    #[serde(rename = "HostName")]
    pub host_name: String,
    #[serde(rename = "Online")]
    pub online: bool,
}

pub fn get_status() -> Result<TailscaleStatus> {
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Tailscale 未运行或命令执行失败");
    }

    let status: TailscaleStatus = serde_json::from_slice(&output.stdout)?;
    Ok(status)
}

pub fn get_device_name(ip: &str) -> Option<String> {
    if let Ok(status) = get_status() {
        for (_, device) in status.peers {
            if device.tailscale_ips.contains(&ip.to_string()) {
                return Some(device.host_name.clone());
            }
        }
    }
    None
}

pub fn is_tailscale_ip(ip: &str) -> bool {
    ip.starts_with("100.")
}
```

**Step 2: 更新 status 命令集成 Tailscale**

`src/commands/status.rs` 追加:
```rust
use crate::core::tailscale;

pub fn handle_status() -> Result<()> {
    let ssh = ssh_manager::check_ssh_server()?;
    let ts = tailscale::get_status().ok();

    let ssh_icon = if ssh.running { "●" } else { "○" };
    let ssh_text = if ssh.running { "Running" } else { "Stopped" };

    println!("SSH Server:  {} {}", ssh_icon, ssh_text);

    if let Some(ts_status) = ts {
        let ts_icon = if ts_status.self_device.online { "●" } else { "○" };
        println!("Tailscale:   {} Connected", ts_icon);
    } else {
        println!("Tailscale:   ○ Not running");
    }

    println!("Phone IP:    (run config show)");
    Ok(())
}
```

**Step 3: 更新 receive 命令识别设备**

在 `src/commands/receive.rs` 的 watch 回调中:
```rust
use crate::core::{config, tailscale};
use crate::core::file_watcher;
use chrono::Local;
use std::path::PathBuf;
use anyhow::Result;

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = dirs::home_dir().unwrap();
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

pub fn handle_receive(watch: bool) -> Result<()> {
    let config = load_config()?;
    let dir = expand_tilde(&config.paths.receive_dir);

    if !std::fs::exists(&dir)? {
        std::fs::create_dir_all(&dir)?;
    }

    if watch {
        println!("📥 接收监控启动");
        println!("");
        file_watcher::watch_directory(&dir, |filename| {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
            let device_name = get_device_display_name(&config, "unknown");
            println!("[{}] 📥 收到文件", now);
            println!("  设备: {}", device_name);
            println!("  文件: {}", filename);
            println!("");
        })?;
    } else {
        println!("📁 已接收文件:");
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            println!("  - {}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}

fn get_device_display_name(config: &config::Config, ip: &str) -> String {
    // 1. 检查别名
    if let Some(alias) = config.aliases.get(ip) {
        return format!("{} ({})", alias, ip);
    }

    // 2. 检查 Tailscale
    if tailscale::is_tailscale_ip(ip) {
        if let Some(name) = tailscale::get_device_name(ip) {
            return format!("{} ({})", name, ip);
        }
    }

    // 3. 直接显示 IP
    ip.to_string()
}
```

**Step 4: 验证**

Run: `cargo run -- status`
Expected: 显示 Tailscale 状态

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 实现 Tailscale 设备识别"
```

---

## Phase 1.11: JSON 输出支持

### Task 11: 添加 --json 选项

**文件:**
- Create: `src/utils/output.rs`
- Modify: `src/utils/mod.rs`
- Modify: `src/commands/status.rs`
- Modify: `src/commands/history.rs`

**Step 1: 实现 output 工具**

`src/utils/output.rs`:
```rust
use serde::Serialize;

pub fn print_json<T: Serialize>(data: &T) {
    println!("{}", serde_json::to_string_pretty(data).unwrap());
}
```

**Step 2: 更新 status 命令**

在 `src/commands/status.rs`:
```rust
use crate::core::{ssh_manager, tailscale};
use crate::utils::output;
use serde::Serialize;
use anyhow::Result;

#[derive(Serialize)]
struct StatusJson {
    ssh_server: SshStatusJson,
    tailscale: TailscaleStatusJson,
}

#[derive(Serialize)]
struct SshStatusJson {
    status: String,
    port: u16,
}

#[derive(Serialize)]
struct TailscaleStatusJson {
    status: String,
    device: Option<String>,
}

pub fn handle_status(json: bool) -> Result<()> {
    let ssh = ssh_manager::check_ssh_server()?;
    let ts = tailscale::get_status().ok();

    if json {
        let data = StatusJson {
            ssh_server: SshStatusJson {
                status: if ssh.running { "running".to_string() } else { "stopped".to_string() },
                port: 22,
            },
            tailscale: TailscaleStatusJson {
                status: if ts.is_some() { "connected".to_string() } else { "not_running".to_string() },
                device: ts.map(|s| s.self_device.host_name),
            },
        };
        output::print_json(&data);
    } else {
        let ssh_icon = if ssh.running { "●" } else { "○" };
        let ssh_text = if ssh.running { "Running" } else { "Stopped" };

        println!("SSH Server:  {} {}", ssh_icon, ssh_text);

        if let Some(ts_status) = ts {
            let ts_icon = if ts_status.self_device.online { "●" } else { "○" };
            println!("Tailscale:   {} Connected", ts_icon);
        } else {
            println!("Tailscale:   ○ Not running");
        }

        println!("Phone IP:    (run config show)");
    }
    Ok(())
}
```

**Step 3: 更新 main.rs**

```rust
Status {
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},

// 在 match 中
Some(Commands::Status { json }) => commands::status::handle_status(json)?,
```

**Step 4: 验证**

Run: `cargo run -- status`
Run: `cargo run -- status --json`
Expected: JSON 输出

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: 添加 --json 输出支持"
```

---

## Phase 1.12: 最终测试

### Task 12: 完整测试

**Step 1: 编译 release 版本**

Run: `cargo build --release`
Expected: 编译成功，生成 `target/release/thru.exe`

**Step 2: 测试所有命令**

```bash
thru --help
thru --version
thru status
thru status --json
thru start
thru stop
thru config show
thru config set-ip 100.118.195.54
thru config set-alias 100.118.195.54 "我的小米15"
thru list
thru send --help
thru receive
thru history
thru history --clear
```

**Step 3: 最终提交**

```bash
git add .
git commit -m "feat: Thru CLI v0.1.0 完整实现"
```

---

## 依赖库清单

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }  # CLI 解析
serde = { version = "1.0", features = ["derive"] }  # 序列化
toml = "0.8"                                         # TOML 解析
notify = "6.1"                                       # 文件监控
serde_json = "1.0"                                   # JSON 处理
chrono = { version = "0.4", features = ["serde"] }   # 时间处理
dirs = "5.0"                                         # 目录获取
anyhow = "1.0"                                       # 错误处理
thiserror = "2.0"                                    # 自定义错误
```

---

## 备注

- 每个 Task 完成后应立即提交
- 遇到编译错误时检查 `cargo check`
- Windows 特定命令使用 PowerShell
- `~` 路径使用 `dirs::home_dir()` 展开