use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, patch};
use axum::{Json, Router};

use rekall_web::{created, no_content};
use crate::catalog::{TaskQuery, TaskRequest, TaskResponse, TaskReviewRequest};
use rekall_web::error::ApiResult;
use rekall_web::extract::{optional_uuid, present, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks", get(list).post(create))
        .route("/api/tasks/{id}", get(get_one).put(update).delete(delete))
        .route("/api/tasks/{id}/review", patch(review))
}

fn validate(request: &TaskRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()
}

async fn list(State(state): State<ApiState>, Query(query): Query<TaskQuery>) -> ApiResult<Json<Vec<TaskResponse>>> {
    let project_id = optional_uuid("projectId", query.project_id.as_deref())?;
    Ok(Json(state.tasks.list(project_id).await?))
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskResponse>> {
    Ok(Json(state.tasks.get(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<TaskRequest>) -> ApiResult<Response> {
    validate(&request)?;
    Ok(created(state.tasks.create(request).await?))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.tasks.update(id, request).await?))
}

async fn review(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskReviewRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    let target = present("reviewState", request.review_state)?;
    Ok(Json(state.tasks.review(id, target, request.note.as_deref()).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.tasks.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
