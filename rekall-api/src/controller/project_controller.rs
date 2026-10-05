use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};

use super::{created, no_content};
use crate::dto::{ProjectRepositoryResponse, ProjectRequest, ProjectResponse};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects", get(list).post(create))
        .route("/api/projects/{id}", get(get_one).put(update).delete(delete))
        .route("/api/projects/{id}/repository", get(repository))
}

fn validate(request: &ProjectRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("title", request.title.as_deref())
        .finish()
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<ProjectResponse>>> {
    Ok(Json(state.projects.list().await?))
}

async fn get_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ProjectResponse>> {
    Ok(Json(state.projects.get(uuid("id", &id)?).await?))
}

/// What the project page's repository strip shows.
async fn repository(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ProjectRepositoryResponse>> {
    Ok(Json(state.projects.repository(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<ProjectRequest>) -> ApiResult<Response> {
    validate(&request)?;
    Ok(created(state.projects.create(request).await?))
}

async fn update(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectRequest>,
) -> ApiResult<Json<ProjectResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.projects.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.projects.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
