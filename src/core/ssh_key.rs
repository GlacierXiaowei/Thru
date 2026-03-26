use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, bail};
use serde::Serialize;

pub fn get_key_path() -> PathBuf {
    super::config::get_config_dir().join("id_ed25519")
}

pub fn get_pub_key_path() -> PathBuf {
    super::config::get_config_dir().join("id_ed25519.pub")
}

pub fn key_exists() -> bool {
    get_key_path().exists()
}

pub fn generate_key() -> Result<()> {
    let key_path = get_key_path();
    
    if key_exists() {
        bail!("SSH 密钥已存在: {}\n使用 --force 覆盖", key_path.display());
    }
    
    let parent = key_path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    
    let output = Command::new("ssh-keygen")
        .args([
            "-t", "ed25519",
            "-f", key_path.to_str().unwrap(),
            "-N", "",
            "-C", "thru@pc"
        ])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("生成密钥失败: {}", stderr);
    }
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
    }
    
    Ok(())
}

pub fn get_public_key() -> Result<String> {
    let pub_path = get_pub_key_path();
    if !pub_path.exists() {
        bail!("公钥文件不存在，请先运行 thru config keygen");
    }
    Ok(std::fs::read_to_string(pub_path)?.trim().to_string())
}

#[derive(Debug, Serialize)]
pub struct KeyInfo {
    pub private_key: String,
    pub public_key: String,
    pub public_key_content: String,
    pub exists: bool,
}

pub fn get_key_info() -> Result<KeyInfo> {
    let exists = key_exists();
    let public_key_content = if exists {
        get_public_key().unwrap_or_default()
    } else {
        String::new()
    };
    
    Ok(KeyInfo {
        private_key: get_key_path().display().to_string(),
        public_key: get_pub_key_path().display().to_string(),
        public_key_content,
        exists,
    })
}