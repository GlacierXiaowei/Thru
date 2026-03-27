use crate::core::config::load_config;
use crate::core::transfer::{self, TransferMethod};
use crate::core::http_client::HttpClient;
use crate::core::discovery::Discovery;
use anyhow::Result;
use std::io::{self, Write};

pub fn handle_send(file: &str, recursive: bool, use_rsync: bool, use_scp: bool, use_lan: Option<Option<String>>, json: bool, no_fallback: bool) -> Result<()> {
    if use_lan.is_some() {
        return handle_lan_send(file, use_lan.and_then(|x| x).as_deref(), json, no_fallback);
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

fn handle_lan_send(file: &str, addr: Option<&str>, json: bool, _no_fallback: bool) -> Result<()> {
    let (ip, port) = if let Some(addr_str) = addr {
        parse_lan_address(addr_str)?
    } else {
        select_device(5)?
    };
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        HttpClient::send_file(&ip, port, file, json, true).await
    })
}

fn parse_lan_address(addr: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = addr.split(':').collect();
    let ip = parts[0].to_string();
    let port: u16 = parts.get(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(53317);
    Ok((ip, port))
}

fn select_device(timeout: u64) -> Result<(String, u16)> {
    println!("🔍 正在搜索设备...");
    
    let devices = Discovery::discover(timeout)?;
    
    if devices.is_empty() {
        anyhow::bail!("未发现任何设备\n提示：请确保目标设备已运行 thru serve");
    }
    
    if devices.len() == 1 {
        let d = &devices[0];
        println!("📡 发现设备: {} ({}:{})", d.name, d.ip, d.port);
        return Ok((d.ip.clone(), d.port));
    }
    
    println!("\n发现的设备:");
    println!("─────────────────────────────────");
    for (i, d) in devices.iter().enumerate() {
        println!("  {}. {} ({}:{}) [{}]", 
            i + 1, d.name, d.ip, d.port, d.network);
    }
    println!("─────────────────────────────────");
    
    print!("选择目标设备 [1-{}]: ", devices.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("无效的选择"))?;
    
    if choice < 1 || choice > devices.len() {
        anyhow::bail!("选择超出范围");
    }
    
    let d = &devices[choice - 1];
    Ok((d.ip.clone(), d.port))
}