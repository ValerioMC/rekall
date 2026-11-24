use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_console::claude::CommandUpdateState;
use rekall_core::claude::Installation;
use crate::shared::error::ApiResult;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/settings/claude", get(status))
        .route("/api/settings/claude/install", post(install))
        .route("/api/settings/claude/commands-update", get(commands_update))
}

async fn status(State(state): State<ApiState>) -> ApiResult<Json<Installation>> {
    Ok(Json(state.claude_install.status().await?))
}

async fn install(State(state): State<ApiState>) -> ApiResult<Json<Installation>> {
    Ok(Json(state.claude_install.install().await?))
}

async fn commands_update(State(state): State<ApiState>) -> Json<CommandUpdateState> {
    Json(state.claude_command_update.state())
}
