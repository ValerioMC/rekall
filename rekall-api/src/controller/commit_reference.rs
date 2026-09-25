//! The console's commit controls: log a task's tip commit, pick one from its recent log, or paste
//! a hash, the same logic `rekall_record_commit` uses.

use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_service::commit::{CommitReferenceView, RecentCommitView};

use crate::dto::{CommitReferenceContextRequest, CommitReferenceDiffResponse, CommitReferenceRequest, PickedCommitReferenceRequest};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, OptionalJsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/commit-references", get(list))
        .route("/api/tasks/{task_id}/commit-references/latest", post(record_latest))
        .route("/api/tasks/{task_id}/recent-commits", get(recent_commits))
        .route("/api/tasks/{task_id}/commit-references", post(record))
        .route("/api/commit-references/{id}/diff", get(diff))
        .route("/api/commit-references/{id}", patch(set_in_context).delete(delete))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<CommitReferenceView>>> {
    Ok(Json(state.services.commit_references.find_all().await?))
}

async fn record_latest(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<CommitReferenceRequest>,
) -> ApiResult<Json<CommitReferenceView>> {
    let task_id = uuid("taskId", &task_id)?;
    let step_id = request.and_then(|r| r.step_id);
    Ok(Json(state.services.commit_references.record_latest_commit(task_id, step_id).await?))
}

/// What the picker lists: the newest commits of the task's project folder, without their diffs.
async fn recent_commits(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<RecentCommitView>>> {
    Ok(Json(state.services.commit_references.recent_commits(uuid("taskId", &task_id)?).await?))
}

/// A commit chosen in the picker or pasted by hand, logged by its hash.
async fn record(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
    JsonBody(request): JsonBody<PickedCommitReferenceRequest>,
) -> ApiResult<Json<CommitReferenceView>> {
    let task_id = uuid("taskId", &task_id)?;
    Validator::new().not_blank("commitHash", request.commit_hash.as_deref()).finish()?;
    Ok(Json(
        state
            .services
            .commit_references
            .record_commit(task_id, request.step_id, request.commit_hash.as_deref())
            .await?,
    ))
}

async fn diff(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<CommitReferenceDiffResponse>> {
    let diff = state.services.commit_references.diff_for(uuid("id", &id)?).await?;
    Ok(Json(CommitReferenceDiffResponse { diff }))
}

/// Whether this commit's hash and diff ride along with `rekall_context`.
async fn set_in_context(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<CommitReferenceContextRequest>,
) -> ApiResult<Json<CommitReferenceView>> {
    let id = uuid("id", &id)?;
    Ok(Json(state.services.commit_references.set_in_context(id, request.in_context).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<()> {
    state.services.commit_references.delete(uuid("id", &id)?).await?;
    Ok(())
}
