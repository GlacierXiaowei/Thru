use crate::core::config::{Config, DeviceConfig, save_config, get_config_path};
use anyhow::Result;

pub fn handle_init(ip: Option<String>, port: Option<u16>, user: Option<String>, json: bool) -> Result<()> {
    let config_path = get_config_path();
    
    let (final_ip, final_port, final_user) = if let (Some(ip), Some(user)) = (&ip, &user) {
        let port = port.unwrap_or(8022);
        (ip.clone(), port, user.clone())
    } else if ip.is_some() || user.is_some() {
        if ip.is_none() {
            anyhow::bail!("请同时提供 --ip 和 --user 参数");
        }
        anyhow::bail!("请同时提供 --ip 和 --user 参数");
    } else {
        println!("📱 配置手机连接信息");
        println!("─────────────────────────────────");
        
        print!("手机 IP: ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        let mut ip_input = String::new();
        io::stdin().read_line(&mut ip_input)?;
        let final_ip = ip_input.trim().to_string();
        
        print!("SSH 端口 [8022]: ");
        io::stdout().flush()?;
        let mut port_input = String::new();
        io::stdin().read_line(&mut port_input)?;
        let final_port = if port_input.trim().is_empty() {
            8022
        } else {
            port_input.trim().parse().unwrap_or(8022)
        };
        
        print!("用户名: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let final_user = user_input.trim().to_string();
        
        (final_ip, final_port, final_user)
    };
    
    if final_ip.is_empty() || final_user.is_empty() {
        anyhow::bail!("IP 和用户名不能为空");
    }
    
    let config = Config {
        device: DeviceConfig {
            phone_ip: final_ip.clone(),
            phone_user: final_user.clone(),
            phone_port: final_port,
        },
        ..Default::default()
    };
    
    save_config(&config)?;
    
    if json {
        println!("{}", serde_json::json!({
            "success": true,
            "config_path": config_path.display().to_string(),
            "device": {
                "ip": final_ip,
                "port": final_port,
                "user": final_user
            }
        }));
    } else {
        println!("─────────────────────────────────");
        println!("✓ 配置完成！");
        println!("  配置文件: {}", config_path.display());
        println!("  手机 IP: {}", final_ip);
        println!("  SSH 端口: {}", final_port);
        println!("  用户名: {}", final_user);
        println!();
        println!("下一步: 运行 thru config keygen 生成 SSH 密钥");
    }
    
    Ok(())
}