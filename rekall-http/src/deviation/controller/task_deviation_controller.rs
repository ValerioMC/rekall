use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::deviation::TaskDeviationResponse;
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/deviations", get(list))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<TaskDeviationResponse>>> {
    let views = state.task_deviations.list_of(uuid("id", &id)?).await?;
    Ok(Json(views.into_iter().map(TaskDeviationResponse::from).collect()))
}
