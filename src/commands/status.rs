use crate::core::{ssh_manager, tailscale};
use crate::utils::output;
use serde::Serialize;
use anyhow::Result;

#[derive(Serialize)]
struct StatusJson {
    ssh_server: SshStatusJson,
    tailscale: TailscaleStatusJson,
}

#[derive(Serialize)]
struct SshStatusJson {
    status: String,
    port: u16,
}

#[derive(Serialize)]
struct TailscaleStatusJson {
    status: String,
    device: Option<String>,
}

pub fn handle_status(json: bool) -> Result<()> {
    let ssh = ssh_manager::check_ssh_server()?;
    let ts = tailscale::get_status().ok();

    if json {
        let data = StatusJson {
            ssh_server: SshStatusJson {
                status: if ssh.running { "running".to_string() } else { "stopped".to_string() },
                port: ssh.port,
            },
            tailscale: TailscaleStatusJson {
                status: if ts.is_some() { "connected".to_string() } else { "not_running".to_string() },
                device: ts.map(|s| s.self_device.host_name),
            },
        };
        output::print_json(&data);
    } else {
        let ssh_icon = if ssh.running { "●" } else { "○" };
        let ssh_text = if ssh.running { "Running" } else { "Stopped" };

        println!("SSH Server:  {} {}", ssh_icon, ssh_text);

        if let Some(ts_status) = ts {
            let ts_icon = if ts_status.self_device.online { "●" } else { "○" };
            println!("Tailscale:   {} Connected", ts_icon);
        } else {
            println!("Tailscale:   ○ Not running");
        }

        println!("Phone IP:    (run config show)");
    }
    Ok(())
}