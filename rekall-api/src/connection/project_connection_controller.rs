use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, OptionalJsonBody};
use rekall_web::no_content;

use crate::connection::{ProjectConnectionRequest, ProjectConnectionResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/connections", get(list))
        .route("/api/projects/{id}/connections/{connection_id}", put(bind).delete(unbind))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectConnectionResponse>>> {
    Ok(Json(state.project_connections.list(uuid("id", &id)?).await?))
}

async fn bind(
    State(state): State<ApiState>,
    Path((id, connection_id)): Path<(String, String)>,
    OptionalJsonBody(request): OptionalJsonBody<ProjectConnectionRequest>,
) -> ApiResult<Json<ProjectConnectionResponse>> {
    let request = request.unwrap_or_default();
    Ok(Json(state.project_connections.bind(uuid("id", &id)?, uuid("connectionId", &connection_id)?, request).await?))
}

async fn unbind(State(state): State<ApiState>, Path((id, connection_id)): Path<(String, String)>) -> ApiResult<Response> {
    state.project_connections.unbind(uuid("id", &id)?, uuid("connectionId", &connection_id)?).await?;
    Ok(no_content())
}
