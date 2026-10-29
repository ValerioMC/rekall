use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::shared::{created, no_content};

use crate::graph::{DeploymentGraphRequest, DeploymentGraphResponse, ReleaseGraphRequest};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/graphs", get(list).post(create))
        .route("/api/graphs/{id}", put(replace).delete(delete))
        .route("/api/projects/{id}/release-graph", put(choose_release))
}

/// The answer is the project's graphs, since choosing one clears the others.
async fn choose_release(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<ReleaseGraphRequest>,
) -> ApiResult<Json<Vec<DeploymentGraphResponse>>> {
    Ok(Json(state.graphs.choose_release(uuid("id", &id)?, request.graph_id).await?.into_iter().map(Into::into).collect()))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<DeploymentGraphResponse>>> {
    Ok(Json(state.graphs.list(uuid("id", &id)?).await?.into_iter().map(Into::into).collect()))
}

async fn create(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<DeploymentGraphRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    validate(&request)?;
    Ok(created(DeploymentGraphResponse::from(state.graphs.create(project_id, request.into()).await?)))
}

async fn replace(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<DeploymentGraphRequest>,
) -> ApiResult<Json<DeploymentGraphResponse>> {
    let id = uuid("id", &id)?;
    validate(&request)?;
    Ok(Json(state.graphs.replace(id, request.into()).await?.into()))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.graphs.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

/// A graph is always written whole, so both the title and the steps have to be there.
fn validate(request: &DeploymentGraphRequest) -> ApiResult<()> {
    Validator::new().not_blank("title", request.title.as_deref()).not_null("steps", &request.steps).finish()
}
