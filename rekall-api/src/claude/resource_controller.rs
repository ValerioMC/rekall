use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::claude::{MemoryAnalysis, ResourceCatalog, ResourceFile};
use rekall_web::error::ApiResult;
use rekall_web::extract::{present, JsonBody};
use rekall_web::no_content;

use super::resource_query::ResourceQuery;
use super::SaveRequest;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/claude/resources", get(catalog))
        .route("/api/claude/resources/analysis", get(analysis))
        .route("/api/claude/resources/file", get(open).put(save).delete(remove))
}

async fn catalog(State(state): State<ApiState>) -> ApiResult<Json<ResourceCatalog>> {
    Ok(Json(state.claude_folder.catalog().await?))
}

async fn analysis(State(state): State<ApiState>) -> ApiResult<Json<MemoryAnalysis>> {
    Ok(Json(state.claude_folder.analyze().await?))
}

async fn open(State(state): State<ApiState>, Query(query): Query<ResourceQuery>) -> ApiResult<Json<ResourceFile>> {
    let path = present("path", query.path)?;
    Ok(Json(state.claude_folder.read(path).await?))
}

async fn save(State(state): State<ApiState>, JsonBody(request): JsonBody<SaveRequest>) -> ApiResult<Json<ResourceFile>> {
    present("path", request.path.as_ref())?;
    present("body", request.body.as_ref())?;
    Ok(Json(state.claude_folder.save(request.into()).await?))
}

async fn remove(State(state): State<ApiState>, Query(query): Query<ResourceQuery>) -> ApiResult<Response> {
    let path = present("path", query.path)?;
    state.claude_folder.delete(path).await?;
    Ok(no_content())
}
