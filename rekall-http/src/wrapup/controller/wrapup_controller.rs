use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_kernel::RekallError;
use rekall_store::WrapupAuthor;
use rekall_core::wrapup::{WrapupVersionDetail, WrapupVersionView, WrapupView};

use crate::shared::no_content;
use crate::wrapup::WrapupRequest;
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/wrapups", get(list))
        .route("/api/tasks/{task_id}/wrapup", get(get_one).put(write).delete(delete))
        .route("/api/tasks/{task_id}/wrapup/versions", get(versions))
        .route("/api/tasks/{task_id}/wrapup/versions/{version_id}", get(version))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<WrapupView>>> {
    Ok(Json(state.wrapups.find_all().await?))
}

async fn get_one(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<WrapupView>> {
    let found = state.wrapups.find(uuid("taskId", &task_id)?).await?;
    Ok(Json(found.ok_or_else(|| RekallError::not_found_msg("This task has no wrapup yet"))?))
}

async fn versions(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<WrapupVersionView>>> {
    Ok(Json(state.wrapups.versions(uuid("taskId", &task_id)?).await?))
}

async fn version(
    State(state): State<ApiState>,
    Path((task_id, version_id)): Path<(String, String)>,
) -> ApiResult<Json<WrapupVersionDetail>> {
    Ok(Json(state.wrapups.version(uuid("taskId", &task_id)?, uuid("versionId", &version_id)?).await?))
}

/// Anything arriving over HTTP was written by hand: the transport stamps the author.
async fn write(State(state): State<ApiState>, Path(task_id): Path<String>, JsonBody(request): JsonBody<WrapupRequest>) -> ApiResult<Json<WrapupView>> {
    let task_id = uuid("taskId", &task_id)?;
    Validator::new().not_blank("bodyMarkdown", request.body_markdown.as_deref()).finish()?;
    let written = state.wrapups.write(task_id, request.body_markdown.as_deref(), WrapupAuthor::Hand).await?;
    Ok(Json(written.wrapup))
}

async fn delete(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Response> {
    state.wrapups.delete(uuid("taskId", &task_id)?).await?;
    Ok(no_content())
}
