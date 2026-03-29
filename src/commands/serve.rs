use crate::core::http_server::HttpServer;
use crate::core::discovery::Discovery;
use anyhow::Result;
use std::thread;
use uuid::Uuid;

pub fn handle_serve(port: Option<u16>, json: bool) -> Result<()> {
    let device_id = Uuid::new_v4().to_string();
    let port = port.unwrap_or(53317);
    
    let discovery_device_id = device_id.clone();
    thread::spawn(move || {
        let _ = Discovery::respond(port, discovery_device_id);
    });
    
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        let server = HttpServer::with_port(port);
        
        if json {
            println!("{}", serde_json::json!({
                "success": true,
                "message": "HTTP server starting",
                "port": port
            }));
        }
        
        server.start().await
    })
}