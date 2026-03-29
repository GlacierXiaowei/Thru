use axum::{
    extract::{Multipart, ConnectInfo},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Result;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use chrono::Local;
use axum::extract::DefaultBodyLimit;

const DEFAULT_PORT: u16 = 53317;
const BACKUP_PORT_1: u16 = 53318;
const BACKUP_PORT_2: u16 = 8080;

fn get_receive_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    Ok(home.join("Downloads").join("Thru"))
}

async fn ensure_receive_dir() -> Result<PathBuf> {
    let dir = get_receive_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).await?;
    }
    Ok(dir)
}

fn is_ip_allowed(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    
    let first: u8 = parts[0].parse().unwrap_or(0);
    let second: u8 = parts[1].parse().unwrap_or(0);
    
    match first {
        10 => true,
        192 => second == 168,
        172 => (16..=31).contains(&second),
        100 => (64..=127).contains(&second),
        127 => true,
        _ => false,
    }
}

pub struct HttpServer {
    port: u16,
}

impl HttpServer {
    pub fn with_port(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/upload", post(upload))
            .route("/files", get(list_files))
            .route("/device", get(device_info))
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024));

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                println!("🌐 HTTP 服务已启动");
                println!("  地址: http://0.0.0.0:{}", self.port);
                l
            }
            Err(_) if self.port == DEFAULT_PORT => {
                println!("⚠ 端口 {} 已被占用，正在使用备用端口 {}...", DEFAULT_PORT, BACKUP_PORT_1);
                let backup_addr = SocketAddr::from(([0, 0, 0, 0], BACKUP_PORT_1));
                TcpListener::bind(backup_addr).await?
            }
            Err(_) if self.port == BACKUP_PORT_1 => {
                println!("⚠ 端口 {}/{} 均不可用，正在使用备用端口 {}...", DEFAULT_PORT, BACKUP_PORT_1, BACKUP_PORT_2);
                let backup_addr = SocketAddr::from(([0, 0, 0, 0], BACKUP_PORT_2));
                TcpListener::bind(backup_addr).await?
            }
            Err(e) => anyhow::bail!("无法绑定任何端口: {}", e),
        };
        
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        ).await?;
        
        Ok(())
    }
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Thru",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

async fn upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    
    if !is_ip_allowed(&client_ip) {
        println!("⛔ 拒绝来自 {} 的上传请求", client_ip);
        return (StatusCode::FORBIDDEN, format!("IP {} not allowed", client_ip)).into_response();
    }
    
    println!("✅ 收到上传请求 from {}", client_ip);
    
    let receive_dir = match ensure_receive_dir().await {
        Ok(d) => d,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };
    
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap_or("unknown").to_string();
        println!("📥 开始接收: {}", name);
        
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                println!("❌ 读取文件失败: {}", e);
                return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
            }
        };
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let safe_name = name.replace(|c: char| !c.is_alphanumeric() && c != '.' && c != '-', "_");
        let filename = format!("{}_{}", timestamp, safe_name);
        let file_path = receive_dir.join(&filename);
        
        match fs::File::create(&file_path).await {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&data).await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                if let Err(e) = file.flush().await {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
        
        println!("📥 完成: {} ({} bytes)", file_path.display(), data.len());
    }
    
    (StatusCode::OK, "OK".to_string()).into_response()
}

async fn list_files() -> Json<serde_json::Value> {
    Json(json!({
        "files": [],
        "total": 0
    }))
}

async fn device_info() -> Json<serde_json::Value> {
    Json(json!({
        "name": hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default(),
        "device_id": uuid::Uuid::new_v4().to_string(),
        "port": DEFAULT_PORT
    }))
}