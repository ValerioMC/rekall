use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::search::SearchHit;

use rekall_web::error::ApiResult;
use crate::ApiState;
use crate::search::SearchQuery;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/search", get(search))
}

async fn search(State(state): State<ApiState>, Query(query): Query<SearchQuery>) -> ApiResult<Json<Vec<SearchHit>>> {
    Ok(Json(state.search.search(Some(query.q.as_deref().unwrap_or(""))).await?))
}
