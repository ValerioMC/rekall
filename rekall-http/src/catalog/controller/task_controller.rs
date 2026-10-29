use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, patch, put};
use axum::{Json, Router};

use crate::shared::{created, no_content};
use crate::catalog::{TaskQuery, TaskReleaseRequest, TaskRequest, TaskResponse, TaskReviewRequest};
use crate::shared::error::ApiResult;
use crate::shared::extract::{optional_uuid, present, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks", get(list).post(create))
        .route("/api/tasks/{id}", get(get_one).put(update).delete(delete))
        .route("/api/tasks/{id}/review", patch(review))
        .route("/api/tasks/{id}/release-skipped", put(skip_release))
}

fn validate(request: &TaskRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()
}

async fn list(State(state): State<ApiState>, Query(query): Query<TaskQuery>) -> ApiResult<Json<Vec<TaskResponse>>> {
    let project_id = optional_uuid("projectId", query.project_id.as_deref())?;
    Ok(Json(state.tasks.list(project_id).await?.into_iter().map(Into::into).collect()))
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskResponse>> {
    Ok(Json(state.tasks.get(uuid("id", &id)?).await?.into()))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<TaskRequest>) -> ApiResult<Response> {
    validate(&request)?;
    Ok(created(TaskResponse::from(state.tasks.create(request.into()).await?)))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.tasks.update(id, request.into()).await?.into()))
}

async fn review(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskReviewRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    let target = present("reviewState", request.review_state)?;
    Ok(Json(state.tasks.review(id, target, request.note.as_deref()).await?.into()))
}

async fn skip_release(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskReleaseRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    let skipped = present("skipped", request.skipped)?;
    Ok(Json(state.tasks.skip_release(id, skipped).await?.into()))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.tasks.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
