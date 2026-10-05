use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_service::section::ProjectSectionView;

use super::{created, no_content};
use crate::dto::{ProjectSectionOrderRequest, ProjectSectionRequest};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/sections", get(list).post(create))
        .route("/api/projects/{id}/sections/order", put(reorder))
        .route("/api/project-sections/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectSectionView>>> {
    Ok(Json(state.services.sections.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(state.sections.create(project_id, request).await?))
}

async fn reorder(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectSectionOrderRequest>,
) -> ApiResult<Json<Vec<ProjectSectionView>>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("sectionIds", &request.section_ids).finish()?;
    Ok(Json(state.sections.reorder(project_id, request).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Json<ProjectSectionView>> {
    let id = uuid("id", &id)?;
    if request.title.is_some() {
        Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    }
    Ok(Json(state.sections.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.sections.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
