use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_core::section::SectionContextView;

use crate::section::SectionContextRequest;
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::shared::{created, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/section-contexts", get(list).post(create))
        .route("/api/section-contexts/{id}", put(rename).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<SectionContextView>>> {
    Ok(Json(state.sections.contexts(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SectionContextRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(state.section_contexts.create(project_id, request.into()).await?))
}

async fn rename(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SectionContextRequest>) -> ApiResult<Json<SectionContextView>> {
    let id = uuid("id", &id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(Json(state.section_contexts.rename(id, request.into()).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.section_contexts.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
