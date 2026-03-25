use std::process::Command;
use anyhow::{Result, bail};

#[derive(Debug)]
pub struct SshStatus {
    pub running: bool,
    pub port: u16,
}

fn check_sshd_exists() -> bool {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Service sshd -ErrorAction SilentlyContinue"])
        .output();
    
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn check_ssh_server() -> Result<SshStatus> {
    if !check_sshd_exists() {
        return Ok(SshStatus {
            running: false,
            port: 22,
        });
    }

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
    if !check_sshd_exists() {
        bail!("SSH Server (sshd) 未安装。\n请运行以下命令安装：\n  Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0");
    }

    let output = Command::new("powershell")
        .args(["-Command", "Start-Service sshd"])
        .output()?;

    if output.status.success() {
        println!("✓ SSH Server 已启动");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("denied") || stderr.contains("权限") {
            bail!("需要管理员权限才能启动 SSH Server。\n请以管理员身份运行 PowerShell。");
        }
        bail!("启动失败: {}", stderr);
    }
    Ok(())
}

pub fn stop_ssh_server() -> Result<()> {
    if !check_sshd_exists() {
        bail!("SSH Server (sshd) 未安装。");
    }

    let output = Command::new("powershell")
        .args(["-Command", "Stop-Service sshd"])
        .output()?;

    if output.status.success() {
        println!("✓ SSH Server 已停止");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("denied") || stderr.contains("权限") {
            bail!("需要管理员权限才能停止 SSH Server。\n请以管理员身份运行 PowerShell。");
        }
        bail!("停止失败: {}", stderr);
    }
    Ok(())
}