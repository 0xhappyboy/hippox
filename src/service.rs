use crate::core::Core;
use crate::global::Config;
use crate::protocols;
use std::sync::Arc;
use tracing::info;

pub struct ServiceConfig {
    pub enable_cli: bool,
    pub enable_tcp: bool,
    pub enable_http: bool,
    pub enable_websocket: bool,
    pub enable_grpc: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            enable_cli: true,
            enable_tcp: true,
            enable_http: true,
            enable_websocket: true,
            enable_grpc: true,
        }
    }
}

pub async fn start(core: Core, config: ServiceConfig) -> anyhow::Result<()> {
    let core = Arc::new(core);
    if config.enable_cli {
        let core_cli = core.clone();
        tokio::spawn(async move {
            info!("Starting CLI interface");
            if let Err(e) = protocols::cli::run_cli(&core_cli).await {
                eprintln!("CLI error: {}", e);
            }
        });
    }
    if config.enable_tcp {
        let core_tcp = (*core).clone();
        tokio::spawn(async move {
            let addr = Config::tcp_address();
            info!("Starting TCP server on {}", addr);
            if let Err(e) = protocols::tcp::run_tcp_server(core_tcp, &addr).await {
                eprintln!("TCP server error: {}", e);
            }
        });
    }
    if config.enable_http {
        let core_http = core.clone();
        tokio::spawn(async move {
            let addr = Config::http_address();
            info!("Starting HTTP server on http://{}", addr);
            if let Err(e) = protocols::http::run_http_server(core_http, &addr).await {
                eprintln!("HTTP server error: {}", e);
            }
        });
    }
    if config.enable_websocket {
        let core_ws = core.clone();
        tokio::spawn(async move {
            let addr = Config::websocket_address();
            info!("Starting WebSocket server on ws://{}", addr);
            if let Err(e) = protocols::websocket::run_websocket_server(core_ws, &addr).await {
                eprintln!("WebSocket server error: {}", e);
            }
        });
    }
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    Ok(())
}
