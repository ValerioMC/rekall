use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::claude::{TerminalTarget, TerminalView};
use crate::shared::extract::{uuid, OptionalJsonBody};
use crate::shared::{no_content, ApiResult};

use crate::terminal::OpenTerminalRequest;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/terminals", get(list))
        .route("/api/tasks/{task_id}/terminals", post(open_on_task))
        .route("/api/projects/{project_id}/terminals", post(open_on_project))
        .route("/api/terminals/{id}", get(get_one).delete(close))
}

async fn list(State(state): State<ApiState>) -> Json<Vec<TerminalView>> {
    Json(state.terminals.list())
}

async fn open_on_task(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<OpenTerminalRequest>,
) -> ApiResult<Response> {
    let target = TerminalTarget::Task(uuid("taskId", &task_id)?);
    open(&state, target, request.unwrap_or_default()).await
}

async fn open_on_project(
    State(state): State<ApiState>,
    Path(project_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<OpenTerminalRequest>,
) -> ApiResult<Response> {
    let target = TerminalTarget::Project(uuid("projectId", &project_id)?);
    open(&state, target, request.unwrap_or_default()).await
}

async fn open(state: &ApiState, target: TerminalTarget, request: OpenTerminalRequest) -> ApiResult<Response> {
    let opened = state.terminals.open(request.command(target)?).await?;
    Ok((StatusCode::CREATED, Json(opened)).into_response())
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TerminalView>> {
    Ok(Json(state.terminals.get(uuid("id", &id)?)?))
}

async fn close(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.terminals.close(uuid("id", &id)?).await?;
    Ok(no_content())
}
