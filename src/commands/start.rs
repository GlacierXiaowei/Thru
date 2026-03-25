use crate::core::ssh_manager;
use anyhow::Result;

pub fn handle_start() -> Result<()> {
    ssh_manager::start_ssh_server()
}