use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use super::no_content;
use crate::dto::ProjectResponse;
use crate::error::ApiResult;
use crate::extract::uuid;
use crate::service::{MASTER_SIDE, MAX_UPLOAD_BYTES};
use crate::ApiState;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";

#[derive(Debug, Deserialize)]
struct IconQuery {
    /// Side in device pixels; the whole master when left out.
    size: Option<u32>,
    /// The image's version: present means the URL names one image for good, so it is cached for good.
    v: Option<String>,
}

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/icon-image", get(read).put(replace).delete(remove))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
}

async fn read(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<IconQuery>) -> ApiResult<Response> {
    let png = state.icons.render(uuid("id", &id)?, query.size.unwrap_or(MASTER_SIDE)).await?;
    let cache = if query.v.is_some() { IMMUTABLE } else { "no-cache" };
    Ok(([(header::CONTENT_TYPE, HeaderValue::from_static("image/png")), (header::CACHE_CONTROL, HeaderValue::from_static(cache))], png).into_response())
}

/// The request body is the image file itself, in any format the server can decode.
async fn replace(State(state): State<ApiState>, Path(id): Path<String>, body: Bytes) -> ApiResult<Json<ProjectResponse>> {
    let id = uuid("id", &id)?;
    state.icons.store(id, body.to_vec()).await?;
    Ok(Json(state.catalog.get_project(id).await?))
}

async fn remove(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.icons.clear(uuid("id", &id)?).await?;
    Ok(no_content())
}
