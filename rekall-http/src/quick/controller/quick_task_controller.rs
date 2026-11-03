use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;

use crate::quick::{QuickTaskRequest, QuickTaskResponse};
use crate::shared::created;
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/projects/{id}/quick-tasks", post(create))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<QuickTaskRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("text", request.text.as_deref()).finish()?;
    Ok(created(QuickTaskResponse::from(state.quick_tasks.create(request.command(project_id)).await?)))
}
