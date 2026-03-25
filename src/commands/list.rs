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