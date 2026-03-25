use std::process::Command;
use crate::core::config::Config;
use anyhow::Result;

pub fn send_file(config: &Config, file_path: &str, recursive: bool) -> Result<()> {
    let dest = format!(
        "{}@{}:{}",
        config.device.phone_user,
        config.device.phone_ip,
        "~/storage/downloads/"
    );

    let mut args = vec![
        "-P".to_string(),
        config.device.phone_port.to_string(),
    ];

    if recursive {
        args.push("-r".to_string());
    }

    args.push(file_path.to_string());
    args.push(dest);

    println!("📤 正在发送 {}...", file_path);

    let output = Command::new("scp")
        .args(&args)
        .output()?;

    if output.status.success() {
        println!("✓ 发送成功");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("✗ 发送失败: {}", stderr);
    }

    Ok(())
}