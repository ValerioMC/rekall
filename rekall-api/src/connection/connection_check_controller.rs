use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use rekall_core::connection::ConnectionCheck;
use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/connections/{id}/check", post(check))
}

async fn check(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ConnectionCheck>> {
    Ok(Json(state.connection_checks.check(uuid("id", &id)?).await?))
}
