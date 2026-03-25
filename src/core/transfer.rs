use std::process::Command;
use std::path::Path;
use std::fs;
use crate::core::config::Config;
use crate::core::history::{self, HistoryEntry, DeviceInfo, FileInfo};
use chrono::Local;
use anyhow::Result;

pub fn send_file(config: &Config, file_path: &str, recursive: bool) -> Result<()> {
    let dest = format!(
        "{}@{}:~/storage/downloads/Thru/",
        config.device.phone_user,
        config.device.phone_ip
    );

    let mut args = vec![
        "-P".to_string(),
        config.device.phone_port.to_string(),
    ];

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest.clone());

    println!("📤 正在发送 {}...", file_path);

    let output = Command::new("scp")
        .args(&args)
        .output()?;

    if output.status.success() {
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("✗ 发送失败: {}", stderr);
    }

    Ok(())
}