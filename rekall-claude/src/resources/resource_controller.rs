use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use rekall_api::controller::no_content;
use rekall_api::error::{ApiError, ApiResult};
use rekall_api::extract::{present, JsonBody};
use rekall_common::{InternalFailure, RekallError, Result};

use crate::ClaudeState;

use super::resource_query::ResourceQuery;
use super::{ResourceCatalog, ResourceFile, SaveRequest};

pub fn routes() -> Router<ClaudeState> {
    Router::new()
        .route("/api/claude/resources", get(catalog))
        .route("/api/claude/resources/file", get(open).put(save).delete(remove))
}

async fn catalog(State(state): State<ClaudeState>) -> ApiResult<Json<ResourceCatalog>> {
    let resources = state.resources.clone();
    Ok(Json(blocking(move || Ok(resources.catalog())).await?))
}

async fn open(State(state): State<ClaudeState>, Query(query): Query<ResourceQuery>) -> ApiResult<Json<ResourceFile>> {
    let path = present("path", query.path)?;
    let resources = state.resources.clone();
    Ok(Json(blocking(move || resources.read(&path)).await?))
}

async fn save(State(state): State<ClaudeState>, JsonBody(request): JsonBody<SaveRequest>) -> ApiResult<Json<ResourceFile>> {
    present("path", request.path.as_ref())?;
    present("body", request.body.as_ref())?;
    let resources = state.resources.clone();
    Ok(Json(blocking(move || resources.save(request)).await?))
}

async fn remove(State(state): State<ClaudeState>, Query(query): Query<ResourceQuery>) -> ApiResult<Response> {
    let path = present("path", query.path)?;
    let resources = state.resources.clone();
    blocking(move || resources.delete(&path)).await?;
    Ok(no_content())
}

/// The disk is read and written with blocking calls, so they run off the async workers.
async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> ApiResult<T> {
    let finished = tokio::task::spawn_blocking(work)
        .await
        .map_err(|failed| RekallError::internal(InternalFailure::Background, failed))?;
    finished.map_err(ApiError::from)
}
