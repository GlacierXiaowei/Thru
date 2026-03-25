use std::process::Command;
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct TailscaleStatus {
    #[serde(rename = "Self")]
    pub self_device: TailscaleDevice,
    #[serde(rename = "Peer")]
    pub peers: std::collections::HashMap<String, TailscaleDevice>,
}

#[derive(Debug, Deserialize)]
pub struct TailscaleDevice {
    #[serde(rename = "DNSName")]
    pub dns_name: String,
    #[serde(rename = "TailscaleIPs")]
    pub tailscale_ips: Vec<String>,
    #[serde(rename = "HostName")]
    pub host_name: String,
    #[serde(rename = "Online")]
    pub online: bool,
}

pub fn get_status() -> Result<TailscaleStatus> {
    let output = Command::new("tailscale")
        .args(["status", "--json"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Tailscale 未运行或命令执行失败");
    }

    let status: TailscaleStatus = serde_json::from_slice(&output.stdout)?;
    Ok(status)
}

pub fn get_device_name(ip: &str) -> Option<String> {
    if let Ok(status) = get_status() {
        for (_, device) in status.peers {
            if device.tailscale_ips.contains(&ip.to_string()) {
                return Some(device.host_name.clone());
            }
        }
    }
    None
}

pub fn is_tailscale_ip(ip: &str) -> bool {
    ip.starts_with("100.")
}