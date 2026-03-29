use anyhow::{Result, bail};
use reqwest::multipart;
use reqwest::Body;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

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
        
        let (tx, rx) = mpsc::channel::<Vec<u8>>(64);
        let bytes_sent = Arc::new(AtomicU64::new(0));
        let bytes_sent_clone = bytes_sent.clone();
        let pb_clone = pb.clone();
        let file_path_owned = file_path.to_string();
        
        let file_task = tokio::spawn(async move {
            let mut file = match File::open(&file_path_owned).await {
                Ok(f) => f,
                Err(_) => return None,
            };
            let mut buf = vec![0u8; 64 * 1024];
            
            loop {
                match file.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = buf[..n].to_vec();
                        bytes_sent_clone.fetch_add(n as u64, Ordering::SeqCst);
                        
                        if let Some(ref pb) = pb_clone {
                            pb.set_position(bytes_sent_clone.load(Ordering::SeqCst));
                        }
                        
                        if tx.send(chunk).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            
            Some(())
        });
        
        let stream = ReceiverStream::new(rx);
        let body = Body::wrap_stream(stream.map(Ok::<_, std::io::Error>));
        
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
        
        let _ = file_task.await;
        
        if let Some(pb) = pb {
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