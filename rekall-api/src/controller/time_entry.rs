use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_service::timeentry::TimeEntryView;

use super::no_content;
use crate::dto::TimeEntryEditRequest;
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/time-entries", get(list))
        .route("/api/tasks/{task_id}/time-entries/start", post(start))
        .route("/api/tasks/{task_id}/time-entries/stop", post(stop))
        .route("/api/time-entries/{id}", patch(edit).delete(delete))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<TimeEntryView>>> {
    Ok(Json(state.services.time_entries.find_all().await?))
}

async fn start(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<TimeEntryView>> {
    Ok(Json(state.services.time_entries.start(uuid("taskId", &task_id)?).await?))
}

async fn stop(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<TimeEntryView>> {
    Ok(Json(state.services.time_entries.stop(uuid("taskId", &task_id)?).await?))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<TimeEntryEditRequest>) -> ApiResult<Json<TimeEntryView>> {
    let id = uuid("id", &id)?;
    Validator::new().not_null("startedAt", &request.started_at).finish()?;
    Ok(Json(state.services.time_entries.edit(id, request.started_at, request.stopped_at).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.services.time_entries.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
