use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_service::search::SearchHit;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/search", get(search))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn search(State(state): State<ApiState>, Query(query): Query<SearchQuery>) -> ApiResult<Json<Vec<SearchHit>>> {
    Ok(Json(state.services.search.search(Some(query.q.as_deref().unwrap_or(""))).await?))
}
