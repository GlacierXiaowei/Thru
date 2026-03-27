use crate::core::http_server::HttpServer;
use anyhow::Result;

pub fn handle_serve(port: Option<u16>, json: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    
    rt.block_on(async {
        let server = if let Some(p) = port {
            HttpServer::with_port(p)
        } else {
            HttpServer::new()
        };
        
        if json {
            println!("{}", serde_json::json!({
                "success": true,
                "message": "HTTP server starting",
                "port": port.unwrap_or(53317)
            }));
        }
        
        server.start().await
    })
}