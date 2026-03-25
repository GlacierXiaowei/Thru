use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub aliases: std::collections::HashMap<String, String>,
    pub paths: PathsConfig,
    pub ssh: SshConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub phone_ip: String,
    pub phone_user: String,
    pub phone_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathsConfig {
    pub receive_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConfig {
    pub auto_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device: DeviceConfig {
                phone_ip: String::new(),
                phone_user: String::new(),
                phone_port: 8022,
            },
            aliases: std::collections::HashMap::new(),
            paths: PathsConfig {
                receive_dir: "~/Downloads/Thru".to_string(),
            },
            ssh: SshConfig {
                auto_start: true,
            },
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".thru")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
