use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;

use rekall_core::connection::{ConnectionCheck, ConnectionCheckService};

pub fn routes(checks: ConnectionCheckService) -> Router {
    Router::new().route("/api/connections/{id}/check", post(check)).with_state(checks)
}

async fn check(State(checks): State<ConnectionCheckService>, Path(id): Path<String>) -> ApiResult<Json<ConnectionCheck>> {
    Ok(Json(checks.check(uuid("id", &id)?).await?))
}
