//! `base` is the folder the picker stands in and `path` what was typed on top of it; both are
//! optional, and with neither the listing is the home folder.

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_kernel::{InternalFailure, RekallError};

use crate::filesystem::DirectoryListingResponse;
use crate::shared::error::ApiResult;
use crate::ApiState;
use crate::filesystem::DirectoryListingQuery;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/filesystem/directory", get(list))
}

async fn list(State(state): State<ApiState>, Query(query): Query<DirectoryListingQuery>) -> ApiResult<Json<DirectoryListingResponse>> {
    let listings = state.directories.clone();
    let listing = tokio::task::spawn_blocking(move || listings.list(query.base.as_deref(), query.path.as_deref()))
        .await
        .map_err(|failed| RekallError::internal(InternalFailure::Background, failed))??;
    Ok(Json(listing.into()))
}
