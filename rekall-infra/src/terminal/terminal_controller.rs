use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rekall_web::extract::{uuid, OptionalJsonBody};
use rekall_web::ApiResult;
use rekall_kernel::RekallError;
use rekall_core::claude::TerminalTarget;

use crate::pty::TerminalView;
use crate::ClaudeState;

use super::{OpenTerminalRequest, TerminalMode};

pub fn routes() -> Router<ClaudeState> {
    Router::new()
        .route("/api/terminals", get(list))
        .route("/api/tasks/{task_id}/terminals", axum::routing::post(open_on_task))
        .route("/api/projects/{project_id}/terminals", axum::routing::post(open_on_project))
        .route("/api/terminals/{id}", get(get_one).delete(close))
}

async fn list(State(state): State<ClaudeState>) -> Json<Vec<TerminalView>> {
    Json(state.terminals.list())
}

async fn open_on_task(
    State(state): State<ClaudeState>,
    Path(task_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<OpenTerminalRequest>,
) -> ApiResult<Response> {
    let target = TerminalTarget::Task(uuid("taskId", &task_id)?);
    let safe = request.unwrap_or_default();
    let mode = safe.mode()?;
    open(&state, target, safe, mode).await
}

/// A project alone has nothing to work or plan, so a terminal on it draws a diagram, records the
/// inventory or assesses an advisory.
async fn open_on_project(
    State(state): State<ClaudeState>,
    Path(project_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<OpenTerminalRequest>,
) -> ApiResult<Response> {
    let target = TerminalTarget::Project(uuid("projectId", &project_id)?);
    let safe = request.unwrap_or_default();
    let mode = safe.mode()?;
    if !mode.works_on_a_project() {
        return Err(RekallError::illegal("A terminal on a project only generates a diagram, records the inventory or assesses an advisory. Open one on a task to work or plan.").into());
    }
    open(&state, target, safe, mode).await
}

async fn open(state: &ClaudeState, target: TerminalTarget, safe: OpenTerminalRequest, mode: TerminalMode) -> ApiResult<Response> {
    let opened = state
        .terminals
        .open(
            target,
            safe.step_id,
            safe.skip_permissions.unwrap_or(false),
            safe.model.as_deref(),
            safe.effort.as_deref(),
            mode,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(opened)).into_response())
}

async fn get_one(State(state): State<ClaudeState>, Path(id): Path<String>) -> ApiResult<Json<TerminalView>> {
    let id = uuid("id", &id)?;
    Ok(Json(state.terminals.get(id).ok_or_else(|| RekallError::not_found_msg(format!("No terminal with id {id}")))?))
}

async fn close(State(state): State<ClaudeState>, Path(id): Path<String>) -> ApiResult<Response> {
    let id = uuid("id", &id)?;
    match state.terminals.close(id, "Closed from the console.").await {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(RekallError::not_found_msg(format!("No terminal with id {id}")).into()),
    }
}
