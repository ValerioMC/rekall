use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody, Validator};
use rekall_web::{created, no_content};

use crate::graph::{DeploymentGraphRequest, DeploymentGraphResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/graphs", get(list).post(create))
        .route("/api/graphs/{id}", put(replace).delete(delete))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<DeploymentGraphResponse>>> {
    Ok(Json(state.graphs.list(uuid("id", &id)?).await?))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<DeploymentGraphRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    validate(&request)?;
    Ok(created(state.graphs.create(project_id, request).await?))
}

async fn replace(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<DeploymentGraphRequest>,
) -> ApiResult<Json<DeploymentGraphResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.graphs.replace(id, request).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.graphs.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

/// A graph is always written whole, so both the title and the steps have to be there.
fn validate(request: &DeploymentGraphRequest) -> ApiResult<()> {
    Validator::new().not_blank("title", request.title.as_deref()).not_null("steps", &request.steps).finish()
}
