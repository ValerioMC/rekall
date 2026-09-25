//! `base` is the folder the picker stands in and `path` what was typed on top of it; both are
//! optional, and with neither the listing is the home folder.

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_common::RekallError;
use serde::Deserialize;

use crate::dto::DirectoryListingResponse;
use crate::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/filesystem/directory", get(list))
}

#[derive(Deserialize)]
struct ListingQuery {
    base: Option<String>,
    path: Option<String>,
}

async fn list(State(state): State<ApiState>, Query(query): Query<ListingQuery>) -> ApiResult<Json<DirectoryListingResponse>> {
    let listings = state.directories.clone();
    let listing = tokio::task::spawn_blocking(move || listings.list(query.base.as_deref(), query.path.as_deref()))
        .await
        .map_err(|failed| RekallError::internal("IllegalStateException", failed))??;
    Ok(Json(listing))
}
