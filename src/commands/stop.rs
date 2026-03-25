use crate::core::ssh_manager;
use anyhow::Result;

pub fn handle_stop() -> Result<()> {
    ssh_manager::stop_ssh_server()
}