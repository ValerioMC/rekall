use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::telemetry::TelemetrySettingsView;

use crate::shared::error::ApiResult;
use crate::shared::extract::JsonBody;
use crate::telemetry::TelemetrySettingsRequest;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/telemetry/settings", get(settings).put(save))
}

async fn settings(State(state): State<ApiState>) -> ApiResult<Json<TelemetrySettingsView>> {
    Ok(Json(state.telemetry.read().await?))
}

async fn save(State(state): State<ApiState>, JsonBody(request): JsonBody<TelemetrySettingsRequest>) -> ApiResult<Json<TelemetrySettingsView>> {
    Ok(Json(state.telemetry.change(request.into()).await?))
}
