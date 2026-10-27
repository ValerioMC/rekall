use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::catalog::EpicActivityResponse;
use crate::ApiState;
use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/epics/{id}/activity", get(activity))
}

async fn activity(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<EpicActivityResponse>>> {
    Ok(Json(state.epics.activity(uuid("id", &id)?).await?))
}
