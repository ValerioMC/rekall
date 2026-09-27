use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rekall_api::extract::{uuid, OptionalJsonBody};
use rekall_api::ApiResult;
use rekall_common::RekallError;

use crate::pty::TerminalView;
use crate::ClaudeState;

use super::OpenTerminalRequest;

pub fn routes() -> Router<ClaudeState> {
    Router::new()
        .route("/api/terminals", get(list))
        .route("/api/tasks/{task_id}/terminals", axum::routing::post(open))
        .route("/api/terminals/{id}", get(get_one).delete(close))
}

async fn list(State(state): State<ClaudeState>) -> Json<Vec<TerminalView>> {
    Json(state.terminals.list())
}

async fn open(
    State(state): State<ClaudeState>,
    Path(task_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<OpenTerminalRequest>,
) -> ApiResult<Response> {
    let task_id = uuid("taskId", &task_id)?;
    let safe = request.unwrap_or_default();
    let mode = safe.mode()?;
    let opened = state
        .terminals
        .open(
            task_id,
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
