use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufReader};
use crate::core::config::Config;
use crate::core::history::{self, HistoryEntry, DeviceInfo, FileInfo};
use crate::core::ssh_key;
use chrono::Local;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub enum TransferMethod {
    Rsync,
    Scp,
    Auto,
}

pub fn check_rsync_available() -> bool {
    Command::new("rsync")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_remote_rsync_available(user: &str, ip: &str, port: u16, key_path: Option<&str>) -> bool {
    let port_str = port.to_string();
    let host = format!("{}@{}", user, ip);
    
    let mut args: Vec<String> = vec!["-p".to_string(), port_str];
    
    if let Some(key) = key_path {
        args.push("-i".to_string());
        args.push(key.to_string());
    }
    
    args.push(host);
    args.push("which rsync".to_string());
    
    Command::new("ssh")
        .args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn send_file_with_progress(
    config: &Config,
    file_path: &str,
    recursive: bool,
    method: TransferMethod,
    json: bool,
) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let key_path = ssh_key::get_key_path();
    let key_exists = ssh_key::key_exists();
    let key_str = if key_exists {
        Some(key_path.display().to_string())
    } else {
        None
    };

    let use_rsync = match method {
        TransferMethod::Rsync => true,
        TransferMethod::Scp => false,
        TransferMethod::Auto => {
            let local = check_rsync_available();
            let remote = check_remote_rsync_available(user, ip, port, key_str.as_deref());
            local && remote
        }
    };

    if use_rsync {
        send_via_rsync(config, file_path, recursive, key_str.as_deref(), json)
    } else {
        send_via_scp(config, file_path, recursive, key_str.as_deref(), json)
    }
}

fn send_via_rsync(
    config: &Config,
    file_path: &str,
    recursive: bool,
    key_path: Option<&str>,
    json: bool,
) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    let dest = format!("{}@{}:~/storage/downloads/Thru/", user, ip);

    let path = Path::new(file_path);
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());
    let file_size = if path.exists() {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    if !json {
        println!("📤 正在发送 {}...", file_name);
    }

    let ssh_cmd = if let Some(key) = key_path {
        format!("ssh -p {} -i {}", port, key)
    } else {
        format!("ssh -p {}", port)
    };

    let mut args = vec![
        "-avz".to_string(),
        "--progress".to_string(),
        "-e".to_string(),
        ssh_cmd,
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

    let pb = if !json && file_size > 0 {
        Some(ProgressBar::new(file_size)
            .with_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("█░")))
    } else {
        None
    };

    for line in reader.lines() {
        let line = line?;
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
        record_history(config, file_path)?;
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

fn send_via_scp(
    config: &Config,
    file_path: &str,
    recursive: bool,
    key_path: Option<&str>,
    json: bool,
) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    let dest_dir = "~/storage/downloads/Thru/";

    if !json {
        println!("📁 确保目标目录存在...");
    }
    
    let mut mkdir_args: Vec<String> = vec!["-p".to_string(), port.to_string()];
    
    if let Some(key) = key_path {
        mkdir_args.push("-i".to_string());
        mkdir_args.push(key.to_string());
    }
    
    mkdir_args.push(format!("{}@{}", user, ip));
    mkdir_args.push("mkdir -p ~/storage/downloads/Thru/".to_string());
    
    let mkdir_status = Command::new("ssh")
        .args(&mkdir_args)
        .status()?;
    
    if !mkdir_status.success() && !json {
        println!("⚠ 无法创建目录，尝试继续发送...");
    }

    let dest = format!("{}@{}:{}", user, ip, dest_dir);

    let mut args = vec!["-P".to_string(), port.to_string()];
    
    if let Some(key) = key_path {
        args.push("-i".to_string());
        args.push(key.to_string());
    }

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest.clone());

    if !json {
        println!("📤 正在发送 {}...", file_path);
    }

    let status = Command::new("scp")
        .args(&args)
        .status()?;

    if status.success() {
        if json {
            let path = Path::new(file_path);
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.to_string());
            let file_size = if path.exists() {
                fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            println!("{}", serde_json::json!({
                "success": true,
                "method": "scp",
                "file": {
                    "name": file_name,
                    "size": file_size
                }
            }));
        } else {
            println!("✓ 发送成功");
        }
        record_history(config, file_path)?;
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

fn record_history(config: &Config, file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let file_size = if path.exists() {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let history_count = history::load_history()?.len();
    let entry = HistoryEntry {
        id: history_count as u64 + 1,
        entry_type: "send".to_string(),
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