use crate::core::Hippox;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
struct ChatRequest {
    input: String,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    matched: bool,
    skill_name: Option<String>,
    response: String,
}

pub async fn run_http_server(hippox: Arc<Hippox>, addr: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/chat", post(chat_handler))
        .route("/skills", get(skills_handler))
        .with_state(hippox);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn chat_handler(
    State(core): State<Arc<Hippox>>,
    Json(req): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let result = core.process(&req.input).await;
    Json(ChatResponse {
        matched: result.matched,
        skill_name: result.skill_name,
        response: result.response,
    })
}

async fn skills_handler(State(core): State<Arc<Hippox>>) -> String {
    core.list_skills()
}
