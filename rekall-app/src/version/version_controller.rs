use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use super::{UpdateChecker, VersionStatus};

#[derive(Deserialize)]
struct VersionQuery {
    #[serde(default)]
    refresh: bool,
}

pub fn routes(checker: Arc<UpdateChecker>) -> Router {
    Router::new().route("/api/version", get(version)).with_state(checker)
}

async fn version(State(checker): State<Arc<UpdateChecker>>, Query(query): Query<VersionQuery>) -> Json<VersionStatus> {
    Json(checker.status(query.refresh).await)
}
