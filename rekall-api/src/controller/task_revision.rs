use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_common::RekallError;
use rekall_model::RevisionKind;
use rekall_service::revision::{RestoredRevision, TaskRevisionView};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};
use crate::extract::{required, uuid};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{task_id}/revisions", get(list))
        .route("/api/tasks/{task_id}/revisions/{revision_id}/restore", post(restore))
}

#[derive(Deserialize)]
struct KindQuery {
    kind: Option<String>,
}

async fn list(State(state): State<ApiState>, Path(task_id): Path<String>, Query(query): Query<KindQuery>) -> ApiResult<Json<Vec<TaskRevisionView>>> {
    let task_id = uuid("taskId", &task_id)?;
    let raw = required(query.kind)?;
    let kind = RevisionKind::value_of(&raw).ok_or_else(|| {
        ApiError::Domain(RekallError::internal(
            "MethodArgumentTypeMismatchException",
            format!(
                "Method parameter 'kind': Failed to convert value of type 'java.lang.String' to required type \
                 'dev.rekall.domain.RevisionKind'; Failed to convert from type [java.lang.String] to type \
                 [@org.springframework.web.bind.annotation.RequestParam dev.rekall.domain.RevisionKind] for value [{raw}]"
            ),
        ))
    })?;
    Ok(Json(state.services.revisions.list(task_id, kind).await?))
}

async fn restore(State(state): State<ApiState>, Path((task_id, revision_id)): Path<(String, String)>) -> ApiResult<Json<RestoredRevision>> {
    let task_id = uuid("taskId", &task_id)?;
    let revision_id = uuid("revisionId", &revision_id)?;
    Ok(Json(state.restorer.restore(task_id, revision_id).await?))
}
