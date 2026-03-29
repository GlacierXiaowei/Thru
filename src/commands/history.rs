use crate::core::history;
use crate::utils::output;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct HistoryItem {
    id: u64,
    entry_type: String,
    timestamp: String,
    file_name: String,
    file_size: u64,
    device: String,
}

#[derive(Debug, Serialize)]
struct HistoryResult {
    entries: Vec<HistoryItem>,
    total: usize,
}

pub fn handle_history(show_all: bool, clear: bool, keep: Option<usize>, json: bool) -> Result<()> {
    if clear {
        history::clear_history()?;
        if json {
            output::print_json(&serde_json::json!({
                "success": true,
                "action": "clear"
            }));
        } else {
            println!("✓ 历史记录已清除");
        }
        return Ok(());
    }

    if let Some(n) = keep {
        history::keep_history(n)?;
        if json {
            output::print_json(&serde_json::json!({
                "success": true,
                "action": "keep",
                "count": n
            }));
        } else {
            println!("✓ 已保留最近 {} 条记录", n);
        }
        return Ok(());
    }

    let entries = history::load_history()?;

    if entries.is_empty() {
        if json {
            output::print_json(&HistoryResult {
                entries: vec![],
                total: 0,
            });
        } else {
            println!("暂无传输记录");
        }
        return Ok(());
    }

    let display_count = if show_all { entries.len() } else { std::cmp::min(50, entries.len()) };
    let start = entries.len() - display_count;

    let history_items: Vec<HistoryItem> = entries[start..]
        .iter()
        .map(|e| HistoryItem {
            id: e.id,
            entry_type: e.entry_type.clone(),
            timestamp: e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            file_name: e.file.name.clone(),
            file_size: e.file.size,
            device: e.device.name.clone(),
        })
        .collect();

    if json {
        output::print_json(&HistoryResult {
            entries: history_items,
            total: entries.len(),
        });
    } else {
        println!("📋 传输历史 (最近 {} 条):\n", display_count);

        for item in &history_items {
            let icon = if item.entry_type == "send" { "📤" } else { "📥" };
            println!("[{}] {} {}", item.timestamp, icon, item.file_name);
            println!("  设备: {}", item.device);
            println!("  大小: {}", format_size(item.file_size));
            println!("");
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