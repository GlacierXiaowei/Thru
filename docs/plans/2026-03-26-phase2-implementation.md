# Phase 2: SSH Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 SSH 方案完整可用，支持免密配置、远程拉取、rsync 优化和 --json 输出

**Architecture:** 新增 init/pull/intro 命令，改造 send 使用 rsync，新增 ssh_key 模块管理密钥，使用 indicatif 显示进度条

**Tech Stack:** Rust, clap, inquire, indicatif, rsync/scp

**Known Issues:**
- 路径引号问题：用户路径含空格时 SSH 命令失败，需要在 send.rs/transfer.rs/pull.rs 中加引号

---

## 进度

| Batch | Tasks | 状态 |
|-------|-------|------|
| Batch 1 | Task 1-3 | ✅ 已完成 |
| Batch 2 | Task 4-6 | ✅ 已完成 |
| Batch 3 | Task 7-10 | ✅ 已完成 |

### 详细进度
- ✅ Task 1: 添加依赖 (indicatif, inquire)
- ✅ Task 2: 创建 SSH 密钥管理模块
- ✅ Task 3: 实现 thru config keygen/key-copy
- ✅ Task 4: 实现 thru init 命令
- ✅ Task 5: 检测 rsync 可用性
- ✅ Task 6: 实现 rsync 传输 + 进度条
- ✅ Task 7: 更新 send 命令支持新参数
- ✅ Task 8: 实现 thru pull 命令
- ✅ Task 9: 其他命令 --json 支持
- ✅ Task 10: 测试和文档更新

### 额外完成（不在原计划中）
- ✅ `thru intro` 命令 (来自 help-improvement-design)
- ✅ send 命令 SSH 密钥支持
- ✅ send 命令 --rsync/--scp/--json 参数
- ✅ intro 命令添加 rsync 安装提示

### 已提交
- `bf72d7f` feat: add intro command and SSH key support in send
- `530a841` feat: add init command and rsync transfer with progress bar
- `5d3fec7` docs: add rsync install tip to intro command

---

## Week 1: 配置体验

### Task 1: 添加依赖

**Files:**
- Modify: `Cargo.toml`

**Step 1: 添加新依赖**

```toml
[dependencies]
# ... 现有依赖 ...
indicatif = "0.17"
inquire = "0.7"
```

**Step 2: 验证依赖下载**

Run: `cargo check`
Expected: 无错误

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add indicatif and inquire dependencies"
```

---

### Task 2: 创建 SSH 密钥管理模块

**Files:**
- Create: `src/core/ssh_key.rs`
- Modify: `src/core/mod.rs`

**Step 1: 创建 ssh_key.rs**

```rust
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, bail};

pub fn get_key_path() -> PathBuf {
    crate::core::config::get_config_dir().join("id_ed25519")
}

pub fn get_pub_key_path() -> PathBuf {
    crate::core::config::get_config_dir().join("id_ed25519.pub")
}

pub fn key_exists() -> bool {
    get_key_path().exists()
}

pub fn generate_key() -> Result<()> {
    let key_path = get_key_path();
    
    if key_exists() {
        bail!("SSH 密钥已存在: {}\n使用 --force 覆盖", key_path.display());
    }
    
    let parent = key_path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    
    let output = Command::new("ssh-keygen")
        .args([
            "-t", "ed25519",
            "-f", key_path.to_str().unwrap(),
            "-N", "",
            "-C", "thru@pc"
        ])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("生成密钥失败: {}", stderr);
    }
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
    }
    
    Ok(())
}

pub fn get_public_key() -> Result<String> {
    let pub_path = get_pub_key_path();
    if !pub_path.exists() {
        bail!("公钥文件不存在，请先运行 thru config keygen");
    }
    Ok(std::fs::read_to_string(pub_path)?.trim().to_string())
}

pub fn get_key_info() -> Result<KeyInfo> {
    Ok(KeyInfo {
        private_key: get_key_path().display().to_string(),
        public_key: get_pub_key_path().display().to_string(),
        public_key_content: get_public_key()?,
        exists: key_exists(),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct KeyInfo {
    pub private_key: String,
    pub public_key: String,
    pub public_key_content: String,
    pub exists: bool,
}
```

**Step 2: 更新 mod.rs**

在 `src/core/mod.rs` 添加：
```rust
pub mod ssh_key;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 4: Commit**

```bash
git add src/core/ssh_key.rs src/core/mod.rs
git commit -m "feat: add SSH key management module"
```

---

### Task 3: 实现 thru config keygen

**Files:**
- Modify: `src/commands/config.rs`
- Modify: `src/main.rs`

**Step 1: 更新 config.rs handle_config**

```rust
use crate::core::config::{load_config, save_config, Config};
use crate::core::ssh_key;
use anyhow::Result;

pub fn handle_keygen(force: bool, json: bool) -> Result<()> {
    if ssh_key::key_exists() && !force {
        if json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": "key_exists",
                "message": "SSH 密钥已存在，使用 --force 覆盖"
            }));
            return Ok(());
        }
        println!("SSH 密钥已存在: {}", ssh_key::get_key_path().display());
        println!("使用 --force 覆盖现有密钥");
        return Ok(());
    }
    
    if force && ssh_key::key_exists() {
        std::fs::remove_file(ssh_key::get_key_path())?;
        std::fs::remove_file(ssh_key::get_pub_key_path())?;
    }
    
    ssh_key::generate_key()?;
    
    let info = ssh_key::get_key_info()?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "private_key": info.private_key,
            "public_key": info.public_key,
            "public_key_content": info.public_key_content
        }))?);
    } else {
        println!("✓ 密钥已生成: {}", info.private_key);
        println!("✓ 公钥位置: {}", info.public_key);
    }
    
    Ok(())
}

pub fn handle_key_copy(json: bool) -> Result<()> {
    let info = ssh_key::get_key_info()?;
    
    if !info.exists {
        anyhow::bail!("请先运行 thru config keygen 生成密钥");
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "method": "manual",
            "public_key": info.public_key_content,
            "instructions": format!("echo \"{}\" >> ~/.ssh/authorized_keys", info.public_key_content)
        }))?);
        return Ok(());
    }
    
    println!("请在手机上执行以下命令：");
    println!("─────────────────────────────────");
    println!("mkdir -p ~/.ssh");
    println!("echo \"{}\" >> ~/.ssh/authorized_keys", info.public_key_content);
    println!("chmod 600 ~/.ssh/authorized_keys");
    println!("─────────────────────────────────");
    println!("\n完成后，即可免密登录！");
    
    Ok(())
}

pub fn handle_config(json: bool) -> Result<()> {
    let config = load_config()?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&config)?);
    } else {
        println!("当前配置：");
        println!("  手机 IP: {}", config.device.phone_ip);
        println!("  SSH 端口: {}", config.device.phone_port);
        println!("  用户名: {}", config.device.phone_user);
    }
    
    Ok(())
}
```

**Step 2: 更新 main.rs 命令定义**

在 `src/main.rs` 的 ConfigArgs 结构体和 Commands 枚举中添加：

```rust
#[derive(clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    command: Option<ConfigCommand>,
}

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// 生成 SSH 密钥对
    Keygen {
        /// 强制覆盖现有密钥
        #[arg(short, long)]
        force: bool,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 部署公钥到手机
    KeyCopy {
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
}
```

更新 match 分支：
```rust
Commands::Config(args) => {
    match args.command {
        Some(ConfigCommand::Keygen { force, json }) => {
            commands::config::handle_keygen(force, json)?;
        }
        Some(ConfigCommand::KeyCopy { json }) => {
            commands::config::handle_key_copy(json)?;
        }
        None => {
            commands::config::handle_config(args.json)?;
        }
    }
}
```

**Step 3: 验证编译和命令**

Run: `cargo build --release`
Run: `./target/release/thru config keygen --help`
Expected: 显示帮助信息

**Step 4: Commit**

```bash
git add src/commands/config.rs src/main.rs
git commit -m "feat: implement thru config keygen and key-copy"
```

---

### Task 4: 实现 thru init 命令

**Files:**
- Create: `src/commands/init.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 创建 init.rs**

```rust
use crate::core::config::{Config, save_config, get_config_path};
use anyhow::Result;
use inquire::{Text, Confirm};

#[derive(Debug, serde::Serialize)]
pub struct InitResult {
    pub success: bool,
    pub config_path: String,
    pub device: DeviceInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct DeviceInfo {
    pub ip: String,
    pub port: u16,
    pub user: String,
}

pub fn handle_init(ip: Option<String>, port: Option<u16>, user: Option<String>, json: bool) -> Result<()> {
    let config_path = get_config_path();
    
    let (final_ip, final_port, final_user) = if ip.is_some() || json {
        let ip = ip.unwrap_or_default();
        let port = port.unwrap_or(8022);
        let user = user.unwrap_or_default();
        (ip, port, user)
    } else {
        let ip = Text::new("手机 IP:")
            .with_placeholder("100.118.195.54")
            .prompt()?;
        
        let port_str = Text::new("SSH 端口:")
            .with_placeholder("8022")
            .prompt()?;
        let port: u16 = port_str.parse().unwrap_or(8022);
        
        let user = Text::new("用户名:")
            .with_placeholder("u0_a406")
            .prompt()?;
        
        (ip, port, user)
    };
    
    let config = Config {
        device: crate::core::config::DeviceConfig {
            phone_ip: final_ip.clone(),
            phone_user: final_user.clone(),
            phone_port: final_port,
        },
        ..Default::default()
    };
    
    save_config(&config)?;
    
    if json {
        let result = InitResult {
            success: true,
            config_path: config_path.display().to_string(),
            device: DeviceInfo {
                ip: final_ip,
                port: final_port,
                user: final_user,
            },
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("✓ 配置完成！");
        println!("  配置文件: {}", config_path.display());
        println!("  手机 IP: {}", final_ip);
        println!("  SSH 端口: {}", final_port);
        println!("  用户名: {}", final_user);
    }
    
    Ok(())
}
```

**Step 2: 更新 commands/mod.rs**

添加：
```rust
pub mod init;
```

**Step 3: 更新 main.rs**

添加命令：
```rust
Init {
    /// 手机 IP 地址
    #[arg(short, long)]
    ip: Option<String>,
    /// SSH 端口
    #[arg(short, long)]
    port: Option<u16>,
    /// SSH 用户名
    #[arg(short, long)]
    user: Option<String>,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},
```

添加 match 分支：
```rust
Commands::Init { ip, port, user, json } => {
    commands::init::handle_init(ip, port, user, json)?;
}
```

**Step 4: 验证编译**

Run: `cargo build --release`
Run: `./target/release/thru init --help`
Expected: 显示帮助信息

**Step 5: Commit**

```bash
git add src/commands/init.rs src/commands/mod.rs src/main.rs
git commit -m "feat: implement thru init command"
```

---

## Week 2: 传输优化

### Task 5: 检测 rsync 可用性

**Files:**
- Modify: `src/core/transfer.rs`

**Step 1: 添加检测函数**

在 transfer.rs 顶部添加：
```rust
use std::process::Command;

pub fn check_rsync_available() -> bool {
    Command::new("rsync")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_remote_rsync_available(user: &str, ip: &str, port: u16) -> bool {
    Command::new("ssh")
        .args([
            "-p", &port.to_string(),
            &format!("{}@{}", user, ip),
            "which rsync"
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

**Step 2: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 3: Commit**

```bash
git add src/core/transfer.rs
git commit -m "feat: add rsync availability check"
```

---

### Task 6: 实现 rsync 传输 + 进度解析

**Files:**
- Modify: `src/core/transfer.rs`

**Step 1: 添加 rsync 发送函数**

```rust
use std::io::{BufRead, BufReader};
use indicatif::{ProgressBar, ProgressStyle};

pub enum TransferMethod {
    Rsync,
    Scp,
}

pub fn send_file_with_progress(
    config: &Config,
    file_path: &str,
    recursive: bool,
    force_method: Option<TransferMethod>,
    json: bool,
) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    let dest_dir = "~/storage/downloads/Thru/";

    // 创建目标目录
    let mkdir_status = Command::new("ssh")
        .args([
            "-p", &port.to_string(),
            &format!("{}@{}", user, ip),
            "mkdir -p ~/storage/downloads/Thru/"
        ])
        .status()?;
    
    if !mkdir_status.success() {
        println!("⚠ 无法创建目录，尝试继续发送...");
    }

    // 决定传输方式
    let use_rsync = match force_method {
        Some(TransferMethod::Rsync) => true,
        Some(TransferMethod::Scp) => false,
        None => {
            let local_rsync = check_rsync_available();
            let remote_rsync = check_remote_rsync_available(user, ip, port);
            local_rsync && remote_rsync
        }
    };

    if use_rsync {
        send_via_rsync(config, file_path, recursive, json)
    } else {
        send_via_scp(config, file_path, recursive, json)
    }
}

fn send_via_rsync(config: &Config, file_path: &str, recursive: bool, json: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    let dest = format!("{}@{}:~/storage/downloads/Thru/", user, ip);

    let path = std::path::Path::new(file_path);
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());
    let file_size = if path.exists() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    if !json {
        println!("📤 正在发送 {}...", file_name);
    }

    let mut args = vec![
        "-avz".to_string(),
        "--progress".to_string(),
        "-e".to_string(),
        format!("ssh -p {}", port),
    ];

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest);

    let mut child = Command::new("rsync")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let pb = if !json {
        Some(ProgressBar::new(file_size)
            .with_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("█░")))
    } else {
        None
    };

    for line in reader.lines() {
        let line = line?;
        
        // 解析 rsync --progress 输出
        // 格式: "    2457600  80%    1.20MB/s    0:00:00"
        if line.contains('%') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Some(pb) = &pb {
                    if let Ok(bytes) = parts[0].parse::<u64>() {
                        pb.set_position(bytes);
                    }
                }
            }
        }
    }

    let status = child.wait()?;

    if let Some(pb) = &pb {
        pb.finish();
    }

    if status.success() {
        if json {
            println!("{}", serde_json::json!({
                "success": true,
                "method": "rsync",
                "file": {
                    "name": file_name,
                    "size": file_size
                }
            }));
        } else {
            println!("✓ 发送成功");
        }
        
        // 记录历史
        record_history(config, file_path, "send")?;
    } else {
        if json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": "transfer_failed"
            }));
        } else {
            println!("✗ 发送失败");
        }
    }

    Ok(())
}

fn send_via_scp(config: &Config, file_path: &str, recursive: bool, json: bool) -> Result<()> {
    // 现有的 scp 实现，添加 json 支持
    // ... (保持原有逻辑，添加 json 输出)
    super::send_file(config, file_path, recursive)
}

fn record_history(config: &Config, file_path: &str, entry_type: &str) -> Result<()> {
    use crate::core::history::{self, HistoryEntry, DeviceInfo, FileInfo};
    use chrono::Local;
    
    let path = std::path::Path::new(file_path);
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let file_size = if path.exists() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let history_count = history::load_history()?.len();
    let entry = HistoryEntry {
        id: history_count as u64 + 1,
        entry_type: entry_type.to_string(),
        timestamp: Local::now(),
        device: DeviceInfo {
            name: config.device.phone_user.clone(),
            alias: config.aliases.get(&config.device.phone_ip).cloned(),
            ip: config.device.phone_ip.clone(),
        },
        file: FileInfo {
            name: file_name,
            size: file_size,
            path: file_path.to_string(),
        },
        status: "success".to_string(),
    };
    
    history::add_entry(entry)?;
    Ok(())
}
```

**Step 2: 验证编译**

Run: `cargo check`
Expected: 无错误

**Step 3: Commit**

```bash
git add src/core/transfer.rs
git commit -m "feat: implement rsync transfer with progress bar"
```

---

### Task 7: 更新 send 命令支持新参数

**Files:**
- Modify: `src/commands/send.rs`
- Modify: `src/main.rs`

**Step 1: 更新 send.rs**

```rust
use crate::core::config::load_config;
use crate::core::transfer::{self, TransferMethod};
use anyhow::Result;

pub fn handle_send(file: &str, recursive: bool, use_rsync: bool, use_scp: bool, json: bool) -> Result<()> {
    let config = load_config()?;
    
    let force_method = if use_rsync {
        Some(TransferMethod::Rsync)
    } else if use_scp {
        Some(TransferMethod::Scp)
    } else {
        None
    };
    
    transfer::send_file_with_progress(&config, file, recursive, force_method, json)
}
```

**Step 2: 更新 main.rs**

修改 Send 命令参数：
```rust
Send {
    /// 要发送的文件或目录
    file: String,
    /// 递归发送目录
    #[arg(short, long)]
    recursive: bool,
    /// 强制使用 rsync
    #[arg(long)]
    rsync: bool,
    /// 强制使用 scp
    #[arg(long)]
    scp: bool,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},
```

更新 match：
```rust
Commands::Send { file, recursive, rsync, scp, json } => {
    commands::send::handle_send(&file, recursive, rsync, scp, json)?;
}
```

**Step 3: 验证编译**

Run: `cargo build --release`

**Step 4: Commit**

```bash
git add src/commands/send.rs src/main.rs
git commit -m "feat: add --rsync, --scp, --json flags to send command"
```

---

## Week 3: 远程拉取

### Task 8: 实现 thru pull 命令

**Files:**
- Create: `src/commands/pull.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: 创建 pull.rs**

```rust
use crate::core::config::load_config;
use crate::core::transfer::TransferMethod;
use anyhow::{Result, bail};
use std::process::Command;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, serde::Serialize)]
pub struct FileListItem {
    pub name: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ListResult {
    pub files: Vec<FileListItem>,
    pub total: usize,
    pub total_size: u64,
}

pub fn handle_pull(file: Option<String>, list: bool, all: bool, output: Option<String>, json: bool) -> Result<()> {
    let config = load_config()?;
    
    if config.device.phone_ip.is_empty() {
        bail!("请先运行 thru init 配置设备信息");
    }
    
    if list {
        return list_remote_files(&config, json);
    }
    
    if all {
        return pull_all_files(&config, output, json);
    }
    
    if let Some(file_name) = file {
        return pull_single_file(&config, &file_name, output, json);
    }
    
    bail!("请指定文件名，或使用 --list 查看可用文件");
}

fn list_remote_files(config: &crate::core::config::Config, json: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let output = Command::new("ssh")
        .args([
            "-p", &port.to_string(),
            &format!("{}@{}", user, ip),
            "ls -la ~/storage/downloads/Thru/ 2>/dev/null || echo 'EMPTY'"
        ])
        .output()?;
    
    let result = String::from_utf8_lossy(&output.stdout);
    
    if result.contains("EMPTY") {
        if json {
            println!("{}", serde_json::to_string_pretty(&ListResult {
                files: vec![],
                total: 0,
                total_size: 0,
            })?);
        } else {
            println!("手机 Thru 目录为空");
        }
        return Ok(());
    }
    
    let mut files = Vec::new();
    let mut total_size = 0u64;
    
    for line in result.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            let name = parts[8].to_string();
            let size: u64 = parts[4].parse().unwrap_or(0);
            total_size += size;
            files.push(FileListItem {
                name,
                size,
                modified: parts[5..8].join(" "),
            });
        }
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&ListResult {
            total: files.len(),
            total_size,
            files,
        })?);
    } else {
        println!("手机 Thru 目录文件：");
        println!("─────────────────────────────────");
        for f in &files {
            println!("  {} ({})", f.name, format_size(f.size));
        }
        println!("─────────────────────────────────");
        println!("共 {} 个文件，总大小 {}", files.len(), format_size(total_size));
    }
    
    Ok(())
}

fn pull_single_file(config: &crate::core::config::Config, file_name: &str, output_dir: Option<String>, json: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let local_dir = output_dir.unwrap_or_else(|| config.paths.receive_dir.clone());
    let local_dir = shellexpand::tilde(&local_dir).to_string();
    
    std::fs::create_dir_all(&local_dir)?;
    
    let remote_path = format!("{}@{}:~/storage/downloads/Thru/{}", user, ip, file_name);
    let local_path = format!("{}/{}", local_dir, file_name);
    
    if !json {
        println!("📥 正在拉取 {}...", file_name);
    }
    
    let status = Command::new("scp")
        .args([
            "-P", &port.to_string(),
            &remote_path,
            &local_path
        ])
        .status()?;
    
    if status.success() {
        if json {
            println!("{}", serde_json::json!({
                "success": true,
                "file": {
                    "name": file_name,
                    "local_path": local_path
                }
            }));
        } else {
            println!("✓ 已保存到: {}", local_path);
        }
    } else {
        if json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": "pull_failed"
            }));
        } else {
            println!("✗ 拉取失败");
        }
    }
    
    Ok(())
}

fn pull_all_files(config: &crate::core::config::Config, output_dir: Option<String>, json: bool) -> Result<()> {
    // 先获取文件列表，再逐个拉取
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let output = Command::new("ssh")
        .args([
            "-p", &port.to_string(),
            &format!("{}@{}", user, ip),
            "ls ~/storage/downloads/Thru/"
        ])
        .output()?;
    
    let result = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    
    if files.is_empty() {
        if json {
            println!("{}", serde_json::json!({"success": true, "files": []}));
        } else {
            println!("没有文件可拉取");
        }
        return Ok(());
    }
    
    for file in &files {
        pull_single_file(config, file, output_dir.clone(), json)?;
    }
    
    Ok(())
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
```

**Step 2: 更新 mod.rs**

```rust
pub mod pull;
```

**Step 3: 更新 main.rs**

添加命令：
```rust
Pull {
    /// 要拉取的文件名
    file: Option<String>,
    /// 列出远程文件
    #[arg(short, long)]
    list: bool,
    /// 拉取全部文件
    #[arg(short, long)]
    all: bool,
    /// 保存目录
    #[arg(short, long)]
    output: Option<String>,
    /// JSON 格式输出
    #[arg(long)]
    json: bool,
},
```

添加 match：
```rust
Commands::Pull { file, list, all, output, json } => {
    commands::pull::handle_pull(file, list, all, output, json)?;
}
```

**Step 4: 验证编译**

Run: `cargo build --release`

**Step 5: Commit**

```bash
git add src/commands/pull.rs src/commands/mod.rs src/main.rs
git commit -m "feat: implement thru pull command"
```

---

### Task 9: 添加其他命令的 --json 支持

**Files:**
- Modify: `src/commands/status.rs`
- Modify: `src/commands/list.rs`
- Modify: `src/commands/history.rs`
- Modify: `src/main.rs`

**Step 1: 更新 status.rs**

添加 json 参数支持：
```rust
pub fn handle_status(json: bool) -> Result<()> {
    // ... 现有逻辑 ...
    // 添加 json 输出分支
}
```

**Step 2: 更新 list.rs**

同上模式添加 json 支持。

**Step 3: 更新 history.rs**

同上模式添加 json 支持。

**Step 4: 更新 main.rs**

为相关命令添加 `--json` 参数。

**Step 5: 验证编译**

Run: `cargo build --release`

**Step 6: Commit**

```bash
git add src/commands/
git commit -m "feat: add --json support to all commands"
```

---

### Task 10: 最终测试和文档更新

**Step 1: 运行完整测试**

```bash
cargo build --release
./target/release/thru --help
./target/release/thru init --help
./target/release/thru config keygen --help
./target/release/thru pull --help
./target/release/thru send --help
```

**Step 2: 更新 README**

添加 Phase 2 新命令说明。

**Step 3: Commit**

```bash
git add .
git commit -m "docs: update README with Phase 2 commands"
```

---

## 验证清单

- [ ] `thru init` 交互模式正常
- [ ] `thru init --ip x.x.x.x --json` 参数模式正常
- [ ] `thru config keygen` 生成密钥
- [ ] `thru config key-copy` 显示公钥
- [ ] `thru send file.jpg` 使用 rsync 传输
- [ ] `thru send file.jpg --scp` 降级到 scp
- [ ] `thru pull --list` 列出远程文件
- [ ] `thru pull photo.jpg` 拉取文件
- [ ] 所有命令 `--json` 输出正常