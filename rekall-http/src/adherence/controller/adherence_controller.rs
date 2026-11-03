use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::adherence::{PlanAdherenceResponse, ProjectAdherenceResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::uuid;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{id}/adherence", get(of_task))
        .route("/api/projects/{id}/adherence", get(of_project))
}

async fn of_task(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<PlanAdherenceResponse>> {
    Ok(Json(state.adherence.of_task(uuid("id", &id)?).await?.into()))
}

async fn of_project(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<ProjectAdherenceResponse>> {
    Ok(Json(state.adherence.of_project(uuid("id", &id)?).await?.into()))
}
