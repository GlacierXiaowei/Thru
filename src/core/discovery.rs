use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;
use local_ip_address::local_ip;

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 12, 34, 56);
const MULTICAST_PORT: u16 = 53317;

fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "0.0.0.0".to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoverMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub device_id: String,
    pub network: String,
}

pub struct Discovery;

impl Discovery {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(timeout_secs: u64) -> Result<Vec<DeviceInfo>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
        
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        
        let discover_msg = DiscoverMessage {
            msg_type: "THRU_DISCOVER".to_string(),
            version: "1.0".to_string(),
        };
        let msg_bytes = serde_json::to_vec(&discover_msg)?;
        
        let dest = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_PORT);
        socket.send_to(&msg_bytes, dest)?;
        
        println!("🔍 正在搜索局域网设备...");
        
        let mut devices = Vec::new();
        let mut buf = [0u8; 4096];
        let start = std::time::Instant::now();
        
        while start.elapsed() < Duration::from_secs(timeout_secs) {
            match socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    if let Ok(device) = serde_json::from_slice::<DeviceInfo>(&buf[..len]) {
                        if device.msg_type == "THRU_RESPONSE" {
                            devices.push(device);
                        }
                    }
                }
                Err(_) => break,
            }
        }
        
        Ok(devices)
    }

    pub fn respond(port: u16, device_id: String) -> Result<()> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), MULTICAST_PORT))?;
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        
        let local_ip = get_local_ip();
        
        let response = DeviceInfo {
            msg_type: "THRU_RESPONSE".to_string(),
            name: hostname,
            ip: local_ip.clone(),
            port,
            device_id,
            network: "lan".to_string(),
        };
        
        println!("📡 设备发现服务已启动 (IP: {})", local_ip);
        
        let mut buf = [0u8; 4096];
        
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    if let Ok(msg) = serde_json::from_slice::<DiscoverMessage>(&buf[..len]) {
                        if msg.msg_type == "THRU_DISCOVER" {
                            let response_bytes = serde_json::to_vec(&response)?;
                            socket.send_to(&response_bytes, addr)?;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }
}