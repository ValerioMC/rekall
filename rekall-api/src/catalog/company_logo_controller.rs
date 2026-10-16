use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};

use rekall_web::{no_content, png};
use crate::catalog::{CompanyResponse, ImageSizeQuery};
use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;
use crate::catalog::{MASTER_SIDE, MAX_UPLOAD_BYTES};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/companies/{id}/logo-image", get(read).put(replace).delete(remove))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
}

async fn read(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<ImageSizeQuery>) -> ApiResult<Response> {
    let bytes = state.logos.render(uuid("id", &id)?, query.size.unwrap_or(MASTER_SIDE)).await?;
    Ok(png(bytes, query.v.is_some()))
}

/// The request body is the image file itself, in any format the server can decode.
async fn replace(State(state): State<ApiState>, Path(id): Path<String>, body: Bytes) -> ApiResult<Json<CompanyResponse>> {
    let id = uuid("id", &id)?;
    state.logos.store(id, body.to_vec()).await?;
    Ok(Json(state.companies.get(id).await?))
}

async fn remove(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.logos.clear(uuid("id", &id)?).await?;
    Ok(no_content())
}
