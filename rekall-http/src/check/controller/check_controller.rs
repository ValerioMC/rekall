use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::check::{CheckOrderRequest, CheckRequest, CheckResponse, CheckResultResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::shared::{created, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/checks", get(list).post(create_for_project))
        .route("/api/projects/{id}/checks/order", put(reorder_for_project))
        .route("/api/steps/{id}/checks", axum::routing::post(create_for_step))
        .route("/api/steps/{id}/checks/order", put(reorder_for_step))
        .route("/api/checks/{id}", put(update).delete(delete))
        .route("/api/tasks/{id}/check-results", get(results_of_task))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<CheckResponse>>> {
    let views = state.checks.list(uuid("id", &id)?).await?;
    Ok(Json(views.into_iter().map(CheckResponse::from).collect()))
}

async fn results_of_task(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<CheckResultResponse>>> {
    let views = state.checks.results_of_task(uuid("id", &id)?).await?;
    Ok(Json(views.into_iter().map(CheckResultResponse::from).collect()))
}

async fn create_for_project(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<CheckRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    validate_new(&request)?;
    Ok(created(CheckResponse::from(state.checks.create_for_project(project_id, request.into()).await?)))
}

async fn create_for_step(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<CheckRequest>) -> ApiResult<Response> {
    let step_id = uuid("id", &id)?;
    validate_new(&request)?;
    Ok(created(CheckResponse::from(state.checks.create_for_step(step_id, request.into()).await?)))
}

async fn reorder_for_project(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<CheckOrderRequest>,
) -> ApiResult<Json<Vec<CheckResponse>>> {
    let project_id = uuid("id", &id)?;
    Validator::new().not_null("checkIds", &request.check_ids).finish()?;
    let views = state.checks.reorder_for_project(project_id, request.into()).await?;
    Ok(Json(views.into_iter().map(CheckResponse::from).collect()))
}

async fn reorder_for_step(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<CheckOrderRequest>,
) -> ApiResult<Json<Vec<CheckResponse>>> {
    let step_id = uuid("id", &id)?;
    Validator::new().not_null("checkIds", &request.check_ids).finish()?;
    let views = state.checks.reorder_for_step(step_id, request.into()).await?;
    Ok(Json(views.into_iter().map(CheckResponse::from).collect()))
}

async fn update(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<CheckRequest>) -> ApiResult<Json<CheckResponse>> {
    Ok(Json(CheckResponse::from(state.checks.update(uuid("id", &id)?, request.into()).await?)))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.checks.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

fn validate_new(request: &CheckRequest) -> ApiResult<()> {
    Validator::new()
        .not_blank("label", request.label.as_deref())
        .not_blank("command", request.command.as_deref())
        .not_null("timeoutSeconds", &request.timeout_seconds)
        .finish()
}
