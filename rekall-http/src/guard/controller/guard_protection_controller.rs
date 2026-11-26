use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::guard::GuardProtection;
use rekall_store::SessionProvider;
use serde::Deserialize;

use crate::shared::error::ApiResult;
use crate::ApiState;

#[derive(Default, Deserialize)]
struct ProtectionQuery {
    #[serde(default)]
    provider: SessionProvider,
}

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/guard/status", get(status))
        .route("/api/guard/install", post(install))
        .route("/api/guard/uninstall", post(uninstall))
}

async fn status(State(state): State<ApiState>, Query(query): Query<ProtectionQuery>) -> ApiResult<Json<GuardProtection>> {
    Ok(Json(state.guard_protection.status(query.provider).await?))
}

async fn install(State(state): State<ApiState>, Query(query): Query<ProtectionQuery>) -> ApiResult<Json<GuardProtection>> {
    Ok(Json(state.guard_protection.install(query.provider).await?))
}

async fn uninstall(State(state): State<ApiState>, Query(query): Query<ProtectionQuery>) -> ApiResult<Json<GuardProtection>> {
    Ok(Json(state.guard_protection.uninstall(query.provider).await?))
}
