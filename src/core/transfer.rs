use std::process::Command;
use std::path::Path;
use std::fs;
use crate::core::config::Config;
use crate::core::history::{self, HistoryEntry, DeviceInfo, FileInfo};
use crate::core::ssh_key;
use chrono::Local;
use anyhow::Result;

pub fn send_file(config: &Config, file_path: &str, recursive: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    let dest_dir = "~/storage/downloads/Thru/";
    
    let key_path = ssh_key::get_key_path();
    let key_exists = ssh_key::key_exists();
    let key_str = key_path.display().to_string();

    println!("📁 确保目标目录存在...");
    let mkdir_args = if key_exists {
        vec![
            "-i".to_string(), key_str.clone(),
            "-p".to_string(), port.to_string(),
            format!("{}@{}", user, ip),
            "mkdir -p ~/storage/downloads/Thru/".to_string()
        ]
    } else {
        vec![
            "-p".to_string(), port.to_string(),
            format!("{}@{}", user, ip),
            "mkdir -p ~/storage/downloads/Thru/".to_string()
        ]
    };
    
    let mkdir_status = Command::new("ssh")
        .args(&mkdir_args)
        .status()?;
    
    if !mkdir_status.success() {
        println!("⚠ 无法创建目录，尝试继续发送...");
    }

    let dest = format!("{}@{}:{}", user, ip, dest_dir);

    let mut args = vec![
        "-P".to_string(),
        port.to_string(),
    ];
    
    if key_exists {
        args.push("-i".to_string());
        args.push(key_str);
    }

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest.clone());

    println!("📤 正在发送 {}...", file_path);

    let status = Command::new("scp")
        .args(&args)
        .status()?;

    if status.success() {
        println!("✓ 发送成功");
        
        // 记录历史
        let path = Path::new(file_path);
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());
        
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
        
        if let Err(e) = history::add_entry(entry) {
            eprintln!("警告: 无法保存历史记录: {}", e);
        }
    } else {
        println!("✗ 发送失败");
    }

    Ok(())
}