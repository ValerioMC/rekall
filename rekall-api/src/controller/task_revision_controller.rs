use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_common::RekallError;
use rekall_model::RevisionKind;
use rekall_service::revision::{RestoredRevision, TaskRevisionView};

use crate::error::{ApiError, ApiResult};
use crate::extract::{required, uuid};
use crate::ApiState;
use crate::dto::RevisionKindQuery;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{task_id}/revisions", get(list))
        .route("/api/tasks/{task_id}/revisions/{revision_id}/restore", post(restore))
}

async fn list(State(state): State<ApiState>, Path(task_id): Path<String>, Query(query): Query<RevisionKindQuery>) -> ApiResult<Json<Vec<TaskRevisionView>>> {
    let task_id = uuid("taskId", &task_id)?;
    let raw = required(query.kind)?;
    let kind = RevisionKind::value_of(&raw).ok_or_else(|| {
        ApiError::Domain(RekallError::illegal(format!("Parameter 'kind' is not a revision kind: {raw}")))
    })?;
    Ok(Json(state.revisions.list(task_id, kind).await?))
}

async fn restore(State(state): State<ApiState>, Path((task_id, revision_id)): Path<(String, String)>) -> ApiResult<Json<RestoredRevision>> {
    let task_id = uuid("taskId", &task_id)?;
    let revision_id = uuid("revisionId", &revision_id)?;
    Ok(Json(state.restorer.restore(task_id, revision_id).await?))
}
