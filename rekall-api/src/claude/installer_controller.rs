use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::claude::Installation;
use rekall_web::error::ApiResult;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/settings/claude", get(status)).route("/api/settings/claude/install", post(install))
}

async fn status(State(state): State<ApiState>) -> Json<Installation> {
    Json(state.claude_install.status().await)
}

async fn install(State(state): State<ApiState>) -> ApiResult<Json<Installation>> {
    Ok(Json(state.claude_install.install().await?))
}
