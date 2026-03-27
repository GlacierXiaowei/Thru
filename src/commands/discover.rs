use crate::core::discovery::Discovery;
use anyhow::Result;

pub fn handle_discover(timeout: u64, json: bool) -> Result<()> {
    let devices = Discovery::discover(timeout)?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "devices": devices,
            "total": devices.len()
        }))?);
    } else {
        if devices.is_empty() {
            println!("未发现任何设备");
            println!("\n提示：请确保目标设备已运行 thru serve 或 Python HTTP 服务");
        } else {
            println!("\n发现的设备：");
            println!("─────────────────────────────────");
            for (i, d) in devices.iter().enumerate() {
                println!("  {}. {} ({}:{}) [{}]", 
                    i + 1, d.name, d.ip, d.port, d.network);
            }
            println!("─────────────────────────────────");
            println!("共发现 {} 台设备", devices.len());
        }
    }
    
    Ok(())
}