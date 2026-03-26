use crate::core::config::{load_config, save_config};
use crate::core::ssh_key;
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

pub fn handle_keygen(force: bool, json: bool) -> Result<()> {
    if ssh_key::key_exists() && !force {
        if json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": "key_exists",
                "message": "SSH 密钥已存在，使用 --force 覆盖"
            }));
            return Ok(());
        }
        println!("SSH 密钥已存在: {}", ssh_key::get_key_path().display());
        println!("使用 --force 覆盖现有密钥");
        return Ok(());
    }
    
    if force && ssh_key::key_exists() {
        let _ = std::fs::remove_file(ssh_key::get_key_path());
        let _ = std::fs::remove_file(ssh_key::get_pub_key_path());
    }
    
    ssh_key::generate_key()?;
    
    let info = ssh_key::get_key_info()?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "private_key": info.private_key,
            "public_key": info.public_key,
            "public_key_content": info.public_key_content
        }))?);
    } else {
        println!("✓ 密钥已生成: {}", info.private_key);
        println!("✓ 公钥位置: {}", info.public_key);
    }
    
    Ok(())
}

pub fn handle_key_copy(json: bool) -> Result<()> {
    let info = ssh_key::get_key_info()?;
    
    if !info.exists {
        anyhow::bail!("请先运行 thru config keygen 生成密钥");
    }
    
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "method": "manual",
            "public_key": info.public_key_content,
            "instructions": format!("echo \"{}\" >> ~/.ssh/authorized_keys", info.public_key_content)
        }))?);
        return Ok(());
    }
    
    println!("请在手机上执行以下命令：");
    println!("─────────────────────────────────");
    println!("mkdir -p ~/.ssh");
    println!("echo \"{}\" >> ~/.ssh/authorized_keys", info.public_key_content);
    println!("chmod 600 ~/.ssh/authorized_keys");
    println!("─────────────────────────────────");
    println!("\n完成后，即可免密登录！");
    
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
