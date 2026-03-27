use crate::core::config::load_config;
use crate::utils::output;
use std::fs;
use anyhow::Result;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct FileItem {
    name: String,
    size: u64,
    is_dir: bool,
}

#[derive(Debug, Serialize)]
struct ListResult {
    directory: String,
    files: Vec<FileItem>,
    total: usize,
    total_size: u64,
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = dirs::home_dir().unwrap();
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

pub fn handle_list(show_hidden: bool, json: bool) -> Result<()> {
    let config = load_config()?;
    let dir = expand_tilde(&config.paths.receive_dir);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        if json {
            output::print_json(&ListResult {
                directory: dir.display().to_string(),
                files: vec![],
                total: 0,
                total_size: 0,
            });
        } else {
            println!("📁 已创建接收目录: {:?}", dir);
        }
        return Ok(());
    }

    let entries = fs::read_dir(&dir)?;
    let mut files = Vec::new();
    let mut total_size = 0u64;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        let size = metadata.len();
        let is_dir = metadata.is_dir();
        
        total_size += size;
        files.push(FileItem { name, size, is_dir });
    }

    if json {
        output::print_json(&ListResult {
            directory: dir.display().to_string(),
            total: files.len(),
            total_size,
            files,
        });
    } else {
        println!("📁 {} 内容:\n", dir.display());

        for f in &files {
            let icon = if f.is_dir { "📁" } else { "  " };
            println!("{}{} ({})", icon, f.name, format_size(f.size));
        }

        if files.is_empty() {
            println!("  (空)");
        } else {
            println!("\n  共 {} 项", files.len());
        }
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