use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::claude::ClaudeModel;

use crate::shared::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/claude/models", get(models))
}

async fn models(State(state): State<ApiState>) -> ApiResult<Json<Vec<ClaudeModel>>> {
    Ok(Json(state.claude_models.models().await?))
}
