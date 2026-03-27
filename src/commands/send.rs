use crate::core::config::load_config;
use crate::core::transfer::{self, TransferMethod};
use anyhow::Result;

pub fn handle_send(file: &str, recursive: bool, use_rsync: bool, use_scp: bool, json: bool) -> Result<()> {
    let config = load_config()?;
    
    let method = if use_rsync {
        TransferMethod::Rsync
    } else if use_scp {
        TransferMethod::Scp
    } else {
        TransferMethod::Auto
    };
    
    transfer::send_file_with_progress(&config, file, recursive, method, json)
}