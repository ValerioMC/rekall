use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_api::error::ApiResult;

use super::{ClaudeCodeInstaller, Installation};

pub fn routes(installer: ClaudeCodeInstaller) -> Router {
    Router::new()
        .route("/api/settings/claude", get(status))
        .route("/api/settings/claude/install", post(install))
        .with_state(installer)
}

async fn status(State(installer): State<ClaudeCodeInstaller>) -> Json<Installation> {
    let answer = tokio::task::spawn_blocking(move || installer.status()).await.expect("status never panics");
    Json(answer)
}

async fn install(State(installer): State<ClaudeCodeInstaller>) -> ApiResult<Json<Installation>> {
    let answer = tokio::task::spawn_blocking(move || installer.install()).await.expect("install never panics");
    Ok(Json(answer?))
}
