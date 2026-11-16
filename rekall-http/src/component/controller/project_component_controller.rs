use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use rekall_core::component::ProjectComponentView;

use crate::shared::{created, no_content};
use crate::component::{ComponentScanResponse, ProjectComponentOrderRequest, ProjectComponentRequest};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/components", get(list).post(create))
        .route("/api/projects/{id}/components/order", put(reorder))
        .route("/api/projects/{id}/components/scan", post(scan))
        .route("/api/project-components/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectComponentView>>> {
    Ok(Json(state.component_catalog.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectComponentRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("folder", request.folder.as_deref()).finish()?;
    Ok(created(state.component_catalog.create(project_id, request.into()).await?))
}

async fn scan(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ComponentScanResponse>> {
    Ok(Json(state.component_scan.scan(uuid("id", &id)?).await?.into()))
}

async fn reorder(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectComponentOrderRequest>,
) -> ApiResult<Json<Vec<ProjectComponentView>>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("componentIds", &request.component_ids).finish()?;
    Ok(Json(state.component_catalog.reorder(project_id, request.into()).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectComponentRequest>) -> ApiResult<Json<ProjectComponentView>> {
    Ok(Json(state.component_catalog.update(uuid("id", &id)?, request.into()).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.component_catalog.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
