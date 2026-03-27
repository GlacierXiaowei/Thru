use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Result;

const DEFAULT_PORT: u16 = 53317;
const BACKUP_PORT_1: u16 = 53318;
const BACKUP_PORT_2: u16 = 8080;

pub struct HttpServer {
    port: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub device_id: String,
    pub port: u16,
}

impl HttpServer {
    pub fn new() -> Self {
        Self { port: DEFAULT_PORT }
    }

    pub fn with_port(port: u16) -> Self {
        Self { port }
    }

    pub async fn start(&self) -> Result<()> {
        let app = Router::new()
            .route("/", get(root))
            .route("/upload", post(upload))
            .route("/files", get(list_files))
            .route("/device", get(device_info));

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

        let actual_port = listener.local_addr()?.port();
        
        axum::serve(listener, app).await?;
        
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

async fn upload(mut multipart: Multipart) -> Result<StatusCode, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        println!("📥 收到文件: {} ({} bytes)", name, data.len());
    }
    
    Ok(StatusCode::OK)
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