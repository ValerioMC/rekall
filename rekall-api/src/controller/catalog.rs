use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde::Deserialize;

use super::{created, no_content};
use crate::dto::{ProjectRepositoryResponse, ProjectRequest, ProjectResponse, TaskRequest, TaskResponse, TaskReviewRequest};
use crate::error::ApiResult;
use crate::extract::{optional_uuid, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/{id}", get(get_project).put(update_project).delete(delete_project))
        .route("/api/projects/{id}/repository", get(get_project_repository))
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/{id}", get(get_task).put(update_task).delete(delete_task))
        .route("/api/tasks/{id}/review", patch(review_task))
}

fn validate_project(request: &ProjectRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()
}

fn validate_task(request: &TaskRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()
}

async fn list_projects(State(state): State<ApiState>) -> ApiResult<Json<Vec<ProjectResponse>>> {
    Ok(Json(state.catalog.list_projects().await?))
}

async fn get_project(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ProjectResponse>> {
    Ok(Json(state.catalog.get_project(uuid("id", &id)?).await?))
}

/// What the project page's repository strip shows.
async fn get_project_repository(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ProjectRepositoryResponse>> {
    Ok(Json(state.catalog.get_project_repository(uuid("id", &id)?).await?))
}

async fn create_project(State(state): State<ApiState>, JsonBody(request): JsonBody<ProjectRequest>) -> ApiResult<Response> {
    validate_project(&request)?;
    Ok(created(state.catalog.create_project(request).await?))
}

async fn update_project(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectRequest>,
) -> ApiResult<Json<ProjectResponse>> {
    let id = uuid("id", &id)?;
    validate_project(&request)?;
    Ok(Json(state.catalog.update_project(id, request).await?))
}

async fn delete_project(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.catalog.delete_project(uuid("id", &id)?).await?;
    Ok(no_content())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskQuery {
    project_id: Option<String>,
}

async fn list_tasks(State(state): State<ApiState>, Query(query): Query<TaskQuery>) -> ApiResult<Json<Vec<TaskResponse>>> {
    let project_id = optional_uuid("projectId", query.project_id.as_deref())?;
    Ok(Json(state.catalog.list_tasks(project_id).await?))
}

async fn get_task(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskResponse>> {
    Ok(Json(state.catalog.get_task(uuid("id", &id)?).await?))
}

async fn create_task(State(state): State<ApiState>, JsonBody(request): JsonBody<TaskRequest>) -> ApiResult<Response> {
    validate_task(&request)?;
    Ok(created(state.catalog.create_task(request).await?))
}

async fn update_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    validate_task(&request)?;
    Ok(Json(state.catalog.update_task(id, request).await?))
}

async fn review_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<TaskReviewRequest>,
) -> ApiResult<Json<TaskResponse>> {
    let id = uuid("id", &id)?;
    Validator::new().not_null("reviewState", &request.review_state).finish()?;
    let target = request.review_state.expect("validated");
    Ok(Json(state.catalog.review_task(id, target, request.note.as_deref()).await?))
}

async fn delete_task(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.catalog.delete_task(uuid("id", &id)?).await?;
    Ok(no_content())
}
