use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{delete, patch, post};
use axum::{Json, Router};
use rekall_service::step::TaskStepView;

use super::created;
use crate::dto::StepLinkRequest;
use crate::error::ApiResult;
use crate::extract::{optional_uuid, uuid, JsonBody, Validator};
use crate::ApiState;

/// The routes between a task's steps and the reset of its visits. Every answer is the task's
/// whole checklist, since a route changes how every step of it reads.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/steps/{id}/links", post(add))
        .route("/api/step-links/{id}", patch(edit).delete(remove))
        .route("/api/tasks/{task_id}/visits", delete(reset))
}

async fn add(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<StepLinkRequest>) -> ApiResult<Response> {
    let from = uuid("id", &id)?;
    Validator::new()
        .not_blank("toStepId", request.to_step_id.as_deref())
        .not_blank("outcome", request.outcome.as_deref())
        .finish()?;
    let to = uuid("toStepId", request.to_step_id.as_deref().unwrap_or_default())?;
    Ok(created(state.step_links.add(from, to, request.outcome.as_deref()).await?))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<StepLinkRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let id = uuid("id", &id)?;
    let to = optional_uuid("toStepId", request.to_step_id.as_deref())?;
    Ok(Json(state.step_links.edit(id, to, request.outcome.as_deref()).await?))
}

async fn remove(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.step_links.delete(uuid("id", &id)?).await?))
}

async fn reset(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.flow.reset(uuid("taskId", &task_id)?).await?))
}
