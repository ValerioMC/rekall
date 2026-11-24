use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_console::section::SectionLayoutView;
use rekall_core::section::ProjectSectionView;

use crate::shared::{created, no_content};
use crate::section::{ProjectSectionRequest, SectionLayoutRequest};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/sections", get(list).post(create))
        .route("/api/projects/{id}/sections/layout", put(arrange))
        .route("/api/project-sections/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectSectionView>>> {
    Ok(Json(state.sections.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(state.section_catalog.create(project_id, request.into()).await?))
}

async fn arrange(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<SectionLayoutRequest>,
) -> ApiResult<Json<SectionLayoutView>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("contexts", &request.contexts).finish()?;
    Ok(Json(state.section_layout.arrange(project_id, request.into()).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectSectionRequest>) -> ApiResult<Json<ProjectSectionView>> {
    let id = uuid("id", &id)?;
    if request.title.is_some() {
        Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    }
    Ok(Json(state.section_catalog.update(id, request.into()).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.section_catalog.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
