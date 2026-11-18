use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::catalog::{EpicConversionResponse, TaskResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/tasks/{id}/epic-conversion", get(preview).post(convert))
}

async fn preview(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<EpicConversionResponse>> {
    Ok(Json(state.epic_conversions.preview(uuid("id", &id)?).await?.into()))
}

async fn convert(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskResponse>> {
    Ok(Json(state.epic_conversions.convert(uuid("id", &id)?).await?.into()))
}
