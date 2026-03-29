use anyhow::{Result, bail};
use reqwest::multipart;
use reqwest::Body;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct HttpClient;

impl HttpClient {
    pub async fn send_file(
        ip: &str,
        port: u16,
        file_path: &str,
        json: bool,
        show_progress: bool,
    ) -> Result<()> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            bail!("文件不存在: {}", file_path);
        }
        
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let file_size = std::fs::metadata(path)?.len();
        
        if !json {
            println!("📤 正在发送 {}...", file_name);
        }
        
        let url = format!("http://{}:{}/upload", ip, port);
        
        let pb = if show_progress && !json {
            Some(crate::utils::progress::create_upload_bar(file_size, &file_name))
        } else {
            None
        };
        
        let file = File::open(path).await?;
        
        let pb_arc = Arc::new(Mutex::new(pb));
        let bytes_sent = Arc::new(Mutex::new(0u64));
        let file_arc = Arc::new(Mutex::new(file));
        
        let body = Body::wrap_stream({
            let pb_arc = pb_arc.clone();
            let bytes_sent = bytes_sent.clone();
            let file_arc = file_arc.clone();
            
            async_stream::stream! {
                let mut buf = vec![0u8; 64 * 1024];
                let mut file = file_arc.lock().await;
                
                loop {
                    match file.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut total = bytes_sent.lock().await;
                            *total += n as u64;
                            
                            if let Some(ref pb) = *pb_arc.lock().await {
                                pb.set_position(*total);
                            }
                            
                            yield Ok::<_, std::io::Error>(buf[..n].to_vec());
                        }
                        Err(e) => {
                            yield Err(e);
                            break;
                        }
                    }
                }
            }
        });
        
        let file_part = multipart::Part::stream_with_length(body, file_size)
            .file_name(file_name.clone())
            .mime_str("application/octet-stream")?;
        
        let form = multipart::Form::new()
            .part("file", file_part);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .multipart(form)
            .send()
            .await?;
        
        if let Some(pb) = pb_arc.lock().await.take() {
            pb.finish();
        }
        
        if response.status().is_success() {
            if json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "method": "http",
                    "file": {
                        "name": file_name,
                        "size": file_size
                    }
                }));
            } else {
                println!("✓ 发送成功");
            }
            Ok(())
        } else {
            bail!("HTTP 发送失败: {}", response.status())
        }
    }
}