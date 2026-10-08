use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_service::component::ProjectComponentView;

use super::{created, no_content};
use crate::dto::{ProjectComponentOrderRequest, ProjectComponentRequest};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/components", get(list).post(create))
        .route("/api/projects/{id}/components/order", put(reorder))
        .route("/api/project-components/{id}", put(update).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectComponentView>>> {
    Ok(Json(state.component_catalog.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectComponentRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_blank("folder", request.folder.as_deref()).finish()?;
    Ok(created(state.component_catalog.create(project_id, request).await?))
}

async fn reorder(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ProjectComponentOrderRequest>,
) -> ApiResult<Json<Vec<ProjectComponentView>>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("componentIds", &request.component_ids).finish()?;
    Ok(Json(state.component_catalog.reorder(project_id, request).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectComponentRequest>) -> ApiResult<Json<ProjectComponentView>> {
    Ok(Json(state.component_catalog.update(uuid("id", &id)?, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.component_catalog.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
