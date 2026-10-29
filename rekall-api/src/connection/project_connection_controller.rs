use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, OptionalJsonBody};
use rekall_web::{created, no_content};

use crate::connection::{ProjectConnectionRequest, ProjectConnectionResponse, RepositoryDiscoveryResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/connections", get(list).post(bind))
        .route("/api/projects/{id}/connections/discover", post(discover))
        .route("/api/projects/{id}/connections/{binding_id}", put(update).delete(unbind))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<ProjectConnectionResponse>>> {
    Ok(Json(state.project_connections.list(uuid("id", &id)?).await?.into_iter().map(ProjectConnectionResponse::from).collect()))
}

async fn bind(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    OptionalJsonBody(request): OptionalJsonBody<ProjectConnectionRequest>,
) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    let request = request.unwrap_or_default();
    let connection_id = request.connection_id()?;
    Ok(created(ProjectConnectionResponse::from(state.project_connections.bind(project_id, connection_id, request.into()).await?)))
}

async fn update(
    State(state): State<ApiState>,
    Path((id, binding_id)): Path<(String, String)>,
    OptionalJsonBody(request): OptionalJsonBody<ProjectConnectionRequest>,
) -> ApiResult<Json<ProjectConnectionResponse>> {
    let request = request.unwrap_or_default();
    Ok(Json(state.project_connections.update(uuid("id", &id)?, uuid("bindingId", &binding_id)?, request.into()).await?.into()))
}

async fn unbind(State(state): State<ApiState>, Path((id, binding_id)): Path<(String, String)>) -> ApiResult<Response> {
    state.project_connections.unbind(uuid("id", &id)?, uuid("bindingId", &binding_id)?).await?;
    Ok(no_content())
}

async fn discover(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<RepositoryDiscoveryResponse>> {
    Ok(Json(state.repository_discovery.discover(uuid("id", &id)?).await?))
}
