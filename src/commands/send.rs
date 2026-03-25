use crate::core::config::load_config;
use crate::core::transfer;
use anyhow::Result;

pub fn handle_send(file: &str, recursive: bool) -> Result<()> {
    let config = load_config()?;
    transfer::send_file(&config, file, recursive)
}