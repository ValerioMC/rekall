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
/// unknown or blank model or effort leaves the account default. A missing `mode` is
/// [`TerminalMode::Work`].
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenTerminalRequest {
    pub step_id: Option<Id>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<TerminalMode>,
}

impl OpenTerminalRequest {
    pub fn mode_or_default(&self) -> TerminalMode {
        self.mode.unwrap_or_default()
    }
}

/// What a terminal is opened to do, which decides the `/rk` line typed into it first. `Work`
/// loads the task and works it; `Plan` has the session propose the task's checklist as drafts
/// and build nothing, so a plan needs no session already running on the task.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TerminalMode {
    #[default]
    Work,
    Plan,
}

impl TerminalMode {
    /// The line the session opens on, for a task anchored by `anchors`.
    pub fn first_line(self, anchors: &str) -> String {
        match self {
            TerminalMode::Work => format!("/rk {anchors}"),
            TerminalMode::Plan => format!("/rk {anchors} plan"),
        }
    }
}

impl std::fmt::Display for TerminalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TerminalMode::Work => "WORK",
            TerminalMode::Plan => "PLAN",
        })
    }
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
        .open(
            task_id,
            safe.step_id,
            safe.skip_permissions.unwrap_or(false),
            safe.model.as_deref(),
            safe.effort.as_deref(),
            safe.mode_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    const ANCHORS: &str = "project:vega task:report-builder";

    #[test]
    fn a_work_terminal_loads_the_task() {
        assert_eq!(TerminalMode::Work.first_line(ANCHORS), "/rk project:vega task:report-builder");
    }

    #[test]
    fn a_plan_terminal_asks_the_session_to_plan_the_task() {
        assert_eq!(TerminalMode::Plan.first_line(ANCHORS), "/rk project:vega task:report-builder plan");
    }

    #[test]
    fn a_request_without_a_mode_opens_a_work_terminal() {
        let request: OpenTerminalRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(request.mode_or_default(), TerminalMode::Work);
        let request: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"PLAN"}"#).unwrap();
        assert_eq!(request.mode_or_default(), TerminalMode::Plan);
    }
}
