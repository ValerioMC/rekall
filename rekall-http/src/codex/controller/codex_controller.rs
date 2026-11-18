use axum::extract::{Query, State};
use serde::Deserialize;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_core::codex::{CodexModel, CodexSetup, CodexUsage};

use crate::{shared::ApiError, ApiState};

pub(crate) fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/codex/models", get(models))
        .route("/api/codex/usage", get(usage))
        .route("/api/settings/codex", get(setup))
        .route("/api/settings/codex/install", post(install))
}

async fn models(State(state): State<ApiState>) -> Result<Json<Vec<CodexModel>>, ApiError> {
    Ok(Json(state.codex.models().await?))
}

#[derive(Deserialize)]
struct UsageQuery {
    #[serde(default)]
    refresh: bool,
}

async fn usage(State(state): State<ApiState>, Query(query): Query<UsageQuery>) -> Json<CodexUsage> {
    Json(state.codex.usage(query.refresh).await)
}

async fn setup(State(state): State<ApiState>) -> Result<Json<CodexSetup>, ApiError> {
    Ok(Json(state.codex.setup().await?))
}

async fn install(State(state): State<ApiState>) -> Result<Json<CodexSetup>, ApiError> {
    Ok(Json(state.codex.install().await?))
}
