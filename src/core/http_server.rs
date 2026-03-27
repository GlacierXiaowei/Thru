use axum::{
    extract::Multipart,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use anyhow::Result;

const DEFAULT_PORT: u16 = 53317;
const BACKUP_PORT_1: u16 = 53318;
const BACKUP_PORT_2: u16 = 8080;

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
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    pub async fn start_receive_mode(&self, json: bool) -> Result<()> {
        let save_dir: PathBuf = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Thru");
        
        std::fs::create_dir_all(&save_dir)?;
        
        let save_dir = Arc::new(save_dir);
        let json_mode = json;
        
        let app = Router::new()
            .route("/", get(upload_form))
            .route("/upload", post(move |multipart: Multipart| {
                let save_dir = save_dir.clone();
                async move {
                    receive_upload(multipart, save_dir, json_mode).await
                }
            }))
            .route("/device", get(device_info));
        
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                if !json {
                    println!("🌐 接收服务已启动");
                    println!("  地址: http://0.0.0.0:{}", self.port);
                    println!("  保存到: ~/Downloads/Thru/");
                    println!("  按 Ctrl+C 停止");
                    println!();
                }
                l
            }
            Err(_) => {
                let backup_addr = SocketAddr::from(([0, 0, 0, 0], BACKUP_PORT_1));
                if !json {
                    println!("⚠ 端口 {} 已被占用，使用端口 {}...", self.port, BACKUP_PORT_1);
                }
                TcpListener::bind(backup_addr).await?
            }
        };
        
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

async fn receive_upload(
    mut multipart: Multipart,
    save_dir: Arc<PathBuf>,
    json: bool,
) -> Result<StatusCode, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        
        let file_path = save_dir.join(&name);
        std::fs::write(&file_path, &data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        if !json {
            println!("📥 收到文件: {} ({} bytes)", name, data.len());
            println!("✓ 已保存到: {}", file_path.display());
            println!();
        } else {
            println!("{}", serde_json::json!({
                "event": "file_received",
                "file": {
                    "name": name,
                    "size": data.len(),
                    "path": file_path.to_string_lossy()
                }
            }));
        }
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

async fn upload_form() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../templates/upload.html"))
}