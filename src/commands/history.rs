use crate::core::history;
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