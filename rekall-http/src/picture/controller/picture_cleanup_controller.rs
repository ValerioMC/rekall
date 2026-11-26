use axum::extract::State;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};

use crate::picture::response::picture_cleanup_response::PictureCleanupResponse;
use crate::shared::error::ApiResult;
use crate::shared::{accepted, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/image-cleanup", post(start).get(status).delete(discard))
        .route("/api/image-cleanup/deletion", post(delete))
}

async fn start(State(state): State<ApiState>) -> ApiResult<Response> {
    state.picture_cleanup.start()?;
    Ok(accepted())
}

async fn status(State(state): State<ApiState>) -> Json<PictureCleanupResponse> {
    Json(state.picture_cleanup.status().into())
}

async fn delete(State(state): State<ApiState>) -> ApiResult<Response> {
    state.picture_cleanup.delete()?;
    Ok(accepted())
}

async fn discard(State(state): State<ApiState>) -> ApiResult<Response> {
    state.picture_cleanup.discard()?;
    Ok(no_content())
}
