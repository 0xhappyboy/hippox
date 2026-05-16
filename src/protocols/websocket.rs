use crate::core::Hippox;
use axum::{
    Router,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::get,
};
use std::sync::Arc;
use tracing::{error, info};

pub async fn run_websocket_server(hippox: Arc<Hippox>, addr: &str) -> anyhow::Result<()> {
    let app = Router::new().route("/ws", get(move |ws| websocket_handler(ws, hippox)));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("WebSocket server listening on ws://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn websocket_handler(ws: WebSocketUpgrade, hippox: Arc<Hippox>) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, hippox))
}

async fn handle_websocket(mut socket: WebSocket, hippox: Arc<Hippox>) {
    info!("WebSocket client connected");
    let welcome = format!(
        "Hippo WebSocket Server\nAvailable skills:\n{}",
        hippox.list_skills()
    );
    if let Err(e) = socket.send(Message::Text(welcome)).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                let input = text.trim();
                info!("Received message: {}", input);
                let result = hippox.process(input).await;
                if result.response == "goodbye" {
                    if let Err(e) = socket.send(Message::Text("Goodbye!".to_string())).await {
                        error!("Failed to send goodbye: {}", e);
                    }
                    break;
                }
                let response = if result.matched {
                    format!("🦛 {}", result.response)
                } else {
                    format!("❌ {}", result.response)
                };
                if let Err(e) = socket.send(Message::Text(response)).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
