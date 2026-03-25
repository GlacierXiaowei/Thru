use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub entry_type: String,
    pub timestamp: DateTime<Local>,
    pub device: DeviceInfo,
    pub file: FileInfo,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub alias: Option<String>,
    pub ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub path: String,
}

fn get_history_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".thru").join("history.json")
}

pub fn load_history() -> Result<Vec<HistoryEntry>> {
    let path = get_history_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let history: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_history(history: &[HistoryEntry]) -> Result<()> {
    let path = get_history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(history)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn add_entry(entry: HistoryEntry) -> Result<()> {
    let mut history = load_history()?;
    history.push(entry);
    save_history(&history)?;
    Ok(())
}

pub fn clear_history() -> Result<()> {
    save_history(&[])?;
    Ok(())
}

pub fn keep_history(n: usize) -> Result<()> {
    let history = load_history()?;
    let len = history.len();
    if len > n {
        let trimmed: Vec<HistoryEntry> = history.into_iter().skip(len - n).collect();
        save_history(&trimmed)?;
    }
    Ok(())
}