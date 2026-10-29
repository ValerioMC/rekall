use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::version::VersionStatus;

use crate::version::VersionQuery;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/version", get(version))
}

async fn version(State(state): State<ApiState>, Query(query): Query<VersionQuery>) -> Json<VersionStatus> {
    Json(state.update_check.status(query.refresh).await)
}
