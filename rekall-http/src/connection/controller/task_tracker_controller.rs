use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};

use crate::catalog::TaskResponse;
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/tracker-sync", post(sync))
}

async fn sync(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskResponse>> {
    Ok(Json(state.task_tracker_sync.sync(uuid("id", &id)?).await?.into()))
}
