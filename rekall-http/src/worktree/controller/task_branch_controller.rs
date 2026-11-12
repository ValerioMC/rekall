use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::worktree::TaskBranchResponse;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/branch", get(view).post(start))
}

async fn view(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Option<TaskBranchResponse>>> {
    Ok(Json(state.task_branches.view(uuid("id", &id)?).await?.map(TaskBranchResponse::from)))
}

async fn start(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Option<TaskBranchResponse>>> {
    Ok(Json(state.task_branches.start(uuid("id", &id)?).await?.map(TaskBranchResponse::from)))
}
