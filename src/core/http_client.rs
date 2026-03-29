use anyhow::{Result, bail};
use reqwest::multipart;
use reqwest::Body;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

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
        
        let url = format!("http://{}:{}/upload", ip, port);
        
        if !json {
            println!("📤 正在发送 {}...", file_name);
        }
        
        let file = File::open(path).await?;
        
        let bytes_sent = Arc::new(AtomicU64::new(0));
        let file_arc = Arc::new(Mutex::new(file));
        let bytes_sent_clone = bytes_sent.clone();
        let file_size_clone = file_size;
        let show = show_progress && !json;
        let file_name_clone = file_name.clone();
        let last_pct = Arc::new(AtomicU64::new(100));
        let last_pct_clone = last_pct.clone();
        
        let body = Body::wrap_stream({
            async_stream::stream! {
                let mut buf = vec![0u8; 64 * 1024];
                let mut file = file_arc.lock().await;
                
                loop {
                    match file.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let sent = bytes_sent_clone.fetch_add(n as u64, Ordering::SeqCst) + n as u64;
                            
                            if show {
                                let pct = sent * 100 / file_size_clone;
                                let last = last_pct_clone.swap(pct, Ordering::SeqCst);
                                if pct != last {
                                    print!("\r📤 {} {}% [{}/{}]", file_name_clone, pct, format_size(sent), format_size(file_size_clone));
                                    use std::io::Write;
                                    std::io::stdout().flush().ok();
                                }
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
        
        let form = multipart::Form::new().part("file", file_part);
        
        let client = reqwest::Client::new();
        let response = client.post(&url).multipart(form).send().await?;
        
        if response.status().is_success() {
            if show {
                println!("\r✓ {} 发送完成 ({})    ", file_name, format_size(file_size));
            } else if !json {
                println!("✓ 发送成功");
            }
            if json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "method": "http",
                    "file": { "name": file_name, "size": file_size }
                }));
            }
            Ok(())
        } else {
            bail!("HTTP 发送失败: {}", response.status())
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}