use std::process::Command;
use anyhow::Result;

#[derive(Debug)]
pub struct SshStatus {
    pub running: bool,
    pub port: u16,
}

pub fn check_ssh_server() -> Result<SshStatus> {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Service sshd | Select-Object -ExpandProperty Status"])
        .output()?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    let running = status == "running";

    Ok(SshStatus {
        running,
        port: 22,
    })
}

pub fn start_ssh_server() -> Result<()> {
    Command::new("powershell")
        .args(["-Command", "Start-Service sshd"])
        .output()?;
    println!("✓ SSH Server 已启动");
    Ok(())
}

pub fn stop_ssh_server() -> Result<()> {
    Command::new("powershell")
        .args(["-Command", "Stop-Service sshd"])
        .output()?;
    println!("✓ SSH Server 已停止");
    Ok(())
}