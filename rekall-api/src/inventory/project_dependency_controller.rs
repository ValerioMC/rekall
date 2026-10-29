use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use rekall_core::inventory::{InventorySummaryView, ProjectDependencyView};
use rekall_web::error::ApiResult;
use rekall_web::extract::uuid;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/projects/{id}/dependencies", get(list)).route("/api/inventories", get(summaries))
}

async fn summaries(State(state): State<ApiState>) -> ApiResult<Json<Vec<InventorySummaryView>>> {
    Ok(Json(state.inventory.summaries().await?))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectDependencyView>>> {
    Ok(Json(state.inventory.list(uuid("id", &id)?).await?))
}
