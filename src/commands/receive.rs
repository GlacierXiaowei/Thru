use crate::core::{config, file_watcher, tailscale};
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
    let cfg = config::load_config()?;
    let dir = expand_tilde(&cfg.paths.receive_dir);

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    if watch {
        println!("📥 接收监控启动");
        println!("");
        file_watcher::watch_directory(&dir, |filename| {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
            let device_name = get_device_display_name(&cfg, "unknown");
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

fn get_device_display_name(cfg: &config::Config, ip: &str) -> String {
    if let Some(alias) = cfg.aliases.get(ip) {
        return format!("{} ({})", alias, ip);
    }

    if tailscale::is_tailscale_ip(ip) {
        if let Some(name) = tailscale::get_device_name(ip) {
            return format!("{} ({})", name, ip);
        }
    }

    ip.to_string()
}