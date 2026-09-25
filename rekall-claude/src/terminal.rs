//! Open, list and close in-app terminals. Bytes do not travel here: a pane connects to the
//! socket at `/api/terminal/{id}/io` once the terminal exists.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rekall_api::extract::{uuid, OptionalJsonBody};
use rekall_api::ApiResult;
use rekall_common::{Id, RekallError};
use serde::Deserialize;

use crate::pty::TerminalView;
use crate::ClaudeState;

/// What the console sends to open a terminal. `stepId`, `model` and `effort` are optional; an
/// unknown or blank model or effort leaves the account default.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenTerminalRequest {
    pub step_id: Option<Id>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

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
    let opened = state
        .terminals
        .open(task_id, safe.step_id, safe.skip_permissions.unwrap_or(false), safe.model.as_deref(), safe.effort.as_deref())
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
