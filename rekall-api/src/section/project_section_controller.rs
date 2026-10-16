use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_service::section::ProjectSectionView;

use rekall_web::{created, no_content};
use crate::section::{ProjectSectionOrderRequest, ProjectSectionRequest};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/sections", get(list).post(create))
        .route("/api/projects/{id}/sections/order", put(reorder))
        .route("/api/project-sections/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectSectionView>>> {
    Ok(Json(state.sections.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(state.section_catalog.create(project_id, request).await?))
}

async fn reorder(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectSectionOrderRequest>,
) -> ApiResult<Json<Vec<ProjectSectionView>>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("sectionIds", &request.section_ids).finish()?;
    Ok(Json(state.section_catalog.reorder(project_id, request).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Json<ProjectSectionView>> {
    let id = uuid("id", &id)?;
    if request.title.is_some() {
        Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    }
    Ok(Json(state.section_catalog.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.section_catalog.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
