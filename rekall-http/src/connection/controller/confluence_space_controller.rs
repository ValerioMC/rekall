use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};

use crate::connection::{ConfluenceImportRequest, ConfluenceImportResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, OptionalJsonBody};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/projects/{id}/connections/{binding_id}/confluence/import", post(import))
}

async fn import(
    State(state): State<ApiState>,
    Path((id, binding_id)): Path<(String, String)>,
    OptionalJsonBody(request): OptionalJsonBody<ConfluenceImportRequest>,
) -> ApiResult<Json<ConfluenceImportResponse>> {
    let request = request.unwrap_or_default();
    Ok(Json(state.confluence_spaces.import(uuid("id", &id)?, uuid("bindingId", &binding_id)?, request.into()).await?.into()))
}
