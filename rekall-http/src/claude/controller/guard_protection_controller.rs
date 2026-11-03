use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::claude::GuardProtection;

use crate::shared::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/guard/status", get(status))
        .route("/api/guard/install", post(install))
        .route("/api/guard/uninstall", post(uninstall))
}

async fn status(State(state): State<ApiState>) -> Json<GuardProtection> {
    Json(state.guard_protection.status().await)
}

async fn install(State(state): State<ApiState>) -> ApiResult<Json<GuardProtection>> {
    Ok(Json(state.guard_protection.install().await?))
}

async fn uninstall(State(state): State<ApiState>) -> ApiResult<Json<GuardProtection>> {
    Ok(Json(state.guard_protection.uninstall().await?))
}
