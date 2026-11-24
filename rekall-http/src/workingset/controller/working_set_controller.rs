use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::shared::error::ApiResult;
use crate::shared::extract::{optional_uuid, uuid, JsonBody};
use crate::workingset::{PlaceResponse, RememberPlaceRequest, WorkingSetResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/working-set", get(view))
        .route("/api/working-set/tasks/{task}", put(open).delete(close))
        .route("/api/working-set/places/{project}", put(remember_place))
}

async fn view(State(state): State<ApiState>) -> ApiResult<Json<WorkingSetResponse>> {
    Ok(Json(state.working_set.view().await?.into()))
}

async fn open(State(state): State<ApiState>, Path(task): Path<String>) -> ApiResult<Json<WorkingSetResponse>> {
    Ok(Json(state.working_set.open(uuid("task", &task)?).await?.into()))
}

async fn close(State(state): State<ApiState>, Path(task): Path<String>) -> ApiResult<Json<PlaceResponse>> {
    Ok(Json(state.working_set.close(uuid("task", &task)?).await?.into()))
}

async fn remember_place(State(state): State<ApiState>, Path(project): Path<String>, JsonBody(request): JsonBody<RememberPlaceRequest>) -> ApiResult<Json<PlaceResponse>> {
    let project_id = uuid("project", &project)?;
    let task_id = optional_uuid("taskId", request.task_id.as_deref())?;
    Ok(Json(state.working_set.remember_place(project_id, task_id).await?.into()))
}
