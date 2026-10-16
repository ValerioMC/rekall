use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_service::context::ContextSize;

use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{task_id}/context-size", get(measure))
}

async fn measure(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<ContextSize>> {
    Ok(Json(state.context_size.measure(uuid("taskId", &task_id)?).await?))
}
