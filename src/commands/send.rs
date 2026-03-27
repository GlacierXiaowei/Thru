use crate::core::config::load_config;
use crate::core::transfer::{self, TransferMethod};
use crate::core::http_client::HttpClient;
use anyhow::Result;

pub fn handle_send(file: &str, recursive: bool, use_rsync: bool, use_scp: bool, use_lan: Option<Option<String>>, json: bool) -> Result<()> {
    if let Some(lan_opt) = use_lan {
        return handle_lan_send(file, lan_opt.as_deref(), json);
    }
    
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

fn handle_lan_send(file: &str, addr: Option<&str>, json: bool) -> Result<()> {
    let (ip, port) = if let Some(addr_str) = addr {
        let parts: Vec<&str> = addr_str.split(':').collect();
        let ip = parts[0].to_string();
        let port: u16 = parts.get(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(53317);
        (ip, port)
    } else {
        use crate::core::discovery::Discovery;
        
        println!("🔍 正在搜索设备...");
        let devices = Discovery::discover(5)?;
        
        if devices.is_empty() {
            anyhow::bail!("未发现任何设备。请确保目标设备已运行 thru serve，或手动指定 IP: thru send file.txt --lan 192.168.1.100:53317");
        }
        
        let device = &devices[0];
        println!("📡 已发现设备: {} ({}:{})", device.name, device.ip, device.port);
        (device.ip.clone(), device.port)
    };
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        HttpClient::send_file(&ip, port, file, json, true).await
    })
}