use crate::core::config::load_config;
use crate::core::ssh_key;
use crate::utils::output;
use anyhow::{Result, bail};
use std::process::Command;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileListItem {
    pub name: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Serialize)]
pub struct ListResult {
    pub files: Vec<FileListItem>,
    pub total: usize,
    pub total_size: u64,
}

#[derive(Debug, Serialize)]
pub struct PullResult {
    pub success: bool,
    pub file: Option<FileInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub local_path: String,
    pub size: u64,
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
    
    bail!("请指定文件名，或使用 --list 查看可用文件，或使用 --all 拉取全部");
}

fn get_local_dir(config: &crate::core::config::Config, output: Option<&str>) -> PathBuf {
    let dir = output.unwrap_or(&config.paths.receive_dir);
    if dir.starts_with("~") {
        let home = dirs::home_dir().unwrap();
        home.join(&dir[2..])
    } else {
        PathBuf::from(dir)
    }
}

fn list_remote_files(config: &crate::core::config::Config, json: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let key_path = ssh_key::get_key_path();
    let key_exists = ssh_key::key_exists();
    
    let mut args: Vec<String> = vec!["-p".to_string(), port.to_string()];
    
    if key_exists {
        args.push("-i".to_string());
        args.push(key_path.display().to_string());
    }
    
    args.push(format!("{}@{}", user, ip));
    args.push("ls -la ~/storage/downloads/Thru/ 2>/dev/null || echo 'EMPTY'".to_string());
    
    let output_result = Command::new("ssh")
        .args(&args)
        .output()?;
    
    let result = String::from_utf8_lossy(&output_result.stdout);
    
    if result.contains("EMPTY") || !output_result.status.success() {
        if json {
            output::print_json(&ListResult {
                files: vec![],
                total: 0,
                total_size: 0,
            });
        } else {
            println!("手机 Thru 目录为空或无法访问");
        }
        return Ok(());
    }
    
    let mut files = Vec::new();
    let mut total_size = 0u64;
    
    for line in result.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            let name = parts[8].to_string();
            if name == "." || name == ".." {
                continue;
            }
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
        output::print_json(&ListResult {
            total: files.len(),
            total_size,
            files,
        });
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
    
    let local_dir = get_local_dir(config, output_dir.as_deref());
    std::fs::create_dir_all(&local_dir)?;
    
    let key_path = ssh_key::get_key_path();
    let key_exists = ssh_key::key_exists();
    
    let remote_path = format!("{}@{}:~/storage/downloads/Thru/{}", user, ip, file_name);
    let local_path = local_dir.join(file_name);
    let local_path_str = local_path.display().to_string();
    
    if !json {
        println!("📥 正在拉取 {}...", file_name);
    }
    
    let mut args: Vec<String> = vec!["-P".to_string(), port.to_string()];
    
    if key_exists {
        args.push("-i".to_string());
        args.push(key_path.display().to_string());
    }
    
    args.push(remote_path.clone());
    args.push(local_path_str.clone());
    
    let status = Command::new("scp")
        .args(&args)
        .status()?;
    
    if status.success() {
        let file_size = if local_path.exists() {
            std::fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        
        if json {
            output::print_json(&PullResult {
                success: true,
                file: Some(FileInfo {
                    name: file_name.to_string(),
                    local_path: local_path_str,
                    size: file_size,
                }),
                error: None,
            });
        } else {
            println!("✓ 已保存到: {}", local_path_str);
        }
        
        record_pull_history(config, file_name, file_size)?;
    } else {
        if json {
            output::print_json(&PullResult {
                success: false,
                file: None,
                error: Some("pull_failed".to_string()),
            });
        } else {
            println!("✗ 拉取失败");
        }
    }
    
    Ok(())
}

fn pull_all_files(config: &crate::core::config::Config, output_dir: Option<String>, json: bool) -> Result<()> {
    let user = &config.device.phone_user;
    let ip = &config.device.phone_ip;
    let port = config.device.phone_port;
    
    let key_path = ssh_key::get_key_path();
    let key_exists = ssh_key::key_exists();
    
    let mut args: Vec<String> = vec!["-p".to_string(), port.to_string()];
    
    if key_exists {
        args.push("-i".to_string());
        args.push(key_path.display().to_string());
    }
    
    args.push(format!("{}@{}", user, ip));
    args.push("ls ~/storage/downloads/Thru/ 2>/dev/null".to_string());
    
    let output_result = Command::new("ssh")
        .args(&args)
        .output()?;
    
    let result = String::from_utf8_lossy(&output_result.stdout);
    let files: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    
    if files.is_empty() {
        if json {
            output::print_json(&serde_json::json!({
                "success": true,
                "files": [],
                "count": 0
            }));
        } else {
            println!("没有文件可拉取");
        }
        return Ok(());
    }
    
    let mut success_count = 0;
    let mut failed_files = Vec::new();
    
    for file in &files {
        match pull_single_file(config, file, output_dir.clone(), json) {
            Ok(_) => success_count += 1,
            Err(_) => failed_files.push(file.to_string()),
        }
    }
    
    if json {
        output::print_json(&serde_json::json!({
            "success": true,
            "files_count": files.len(),
            "success_count": success_count,
            "failed_files": failed_files
        }));
    } else {
        println!("\n✓ 拉取完成: {}/{} 个文件", success_count, files.len());
        if !failed_files.is_empty() {
            println!("失败文件: {:?}", failed_files);
        }
    }
    
    Ok(())
}

fn record_pull_history(config: &crate::core::config::Config, file_name: &str, file_size: u64) -> Result<()> {
    use crate::core::history::{self, HistoryEntry, DeviceInfo, FileInfo};
    use chrono::Local;
    
    let history_count = history::load_history()?.len();
    let entry = HistoryEntry {
        id: history_count as u64 + 1,
        entry_type: "pull".to_string(),
        timestamp: Local::now(),
        device: DeviceInfo {
            name: config.device.phone_user.clone(),
            alias: config.aliases.get(&config.device.phone_ip).cloned(),
            ip: config.device.phone_ip.clone(),
        },
        file: FileInfo {
            name: file_name.to_string(),
            size: file_size,
            path: file_name.to_string(),
        },
        status: "success".to_string(),
    };
    
    history::add_entry(entry)?;
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