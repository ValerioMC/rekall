use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};

use super::{created, no_content};
use crate::dto::{TagRequest, TagResponse};
use crate::error::ApiResult;
use crate::extract::{uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tags", get(list).post(create))
        .route("/api/tags/{id}", put(update).delete(delete))
}

fn validate(request: &TagRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("name", request.name.as_deref())
        .not_blank("icon", request.icon.as_deref())
        .not_blank("color", request.color.as_deref())
        .finish()
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<TagResponse>>> {
    Ok(Json(state.tags.list().await?))
}

async fn create(State(state): State<ApiState>, JsonBody(request): JsonBody<TagRequest>) -> ApiResult<Response> {
    validate(&request)?;
    Ok(created(state.tags.create(request).await?))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<TagRequest>) -> ApiResult<Json<TagResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.tags.update(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.tags.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
