use crate::core::config::{load_config, save_config};
use anyhow::Result;

pub fn handle_show() -> Result<()> {
    let config = load_config()?;
    println!("📱 设备配置:");
    println!("  IP: {}", config.device.phone_ip);
    println!("  用户：{}", config.device.phone_user);
    println!("  端口：{}", config.device.phone_port);
    println!("\n📁 路径:");
    println!("  接收目录：{}", config.paths.receive_dir);
    println!("\n🔧 SSH:");
    println!("  自动启动：{}", config.ssh.auto_start);
    if !config.aliases.is_empty() {
        println!("\n🏷️  设备别名:");
        for (ip, alias) in &config.aliases {
            println!("  {} → {}", ip, alias);
        }
    }
    Ok(())
}

pub fn handle_set_ip(ip: &str) -> Result<()> {
    let mut config = load_config()?;
    config.device.phone_ip = ip.to_string();
    save_config(&config)?;
    println!("✓ 手机 IP 已设置为：{}", ip);
    Ok(())
}

pub fn handle_get_ip() -> Result<()> {
    let config = load_config()?;
    println!("{}", config.device.phone_ip);
    Ok(())
}

pub fn handle_set_alias(ip: &str, alias: &str) -> Result<()> {
    let mut config = load_config()?;
    config.aliases.insert(ip.to_string(), alias.to_string());
    save_config(&config)?;
    println!("✓ 设备别名已设置：{} → {}", ip, alias);
    Ok(())
}

pub fn handle_set_user(user: &str) -> Result<()> {
    let mut config = load_config()?;
    config.device.phone_user = user.to_string();
    save_config(&config)?;
    println!("✓ 手机用户已设置为：{}", user);
    Ok(())
}
