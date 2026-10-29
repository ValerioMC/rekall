use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{delete, patch, post};
use axum::{Json, Router};
use rekall_core::flow::RouteTarget;
use rekall_core::step::{FlowPlace, TaskStepView, VisitBudget};

use crate::shared::created;
use crate::flow::{StepFlowPlaceRequest, StepLinkRequest, StepMaxVisitsRequest};
use crate::shared::error::ApiResult;
use crate::shared::extract::{optional_uuid, uuid, JsonBody, Validator};
use crate::ApiState;

/// The routes between a task's steps and the reset of its visits. Every answer is the task's
/// whole checklist, since a route changes how every step of it reads.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/steps/{id}/links", post(add))
        .route("/api/step-links/{id}", patch(edit).delete(remove))
        .route("/api/tasks/{task_id}/visits", delete(reset))
        .route("/api/steps/{id}/flow-place", patch(place))
        .route("/api/steps/{id}/max-visits", patch(limit_visits))
        .route("/api/tasks/{task_id}/flow-places", delete(clear_places))
}

async fn add(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<StepLinkRequest>) -> ApiResult<Response> {
    let from = uuid("id", &id)?;
    let mut validator = Validator::new().not_blank("outcome", request.outcome.as_deref());
    if !request.ends_flow {
        validator = validator.not_blank("toStepId", request.to_step_id.as_deref());
    }
    validator.finish()?;
    let target = route_target(&request)?.unwrap_or(RouteTarget::End);
    Ok(created(state.step_links.add(from, target, request.outcome.as_deref()).await?))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<StepLinkRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let id = uuid("id", &id)?;
    Ok(Json(state.step_links.edit(id, route_target(&request)?, request.outcome.as_deref()).await?))
}

/// The end of the flow when the request says so, the step it names otherwise, nothing when neither.
fn route_target(request: &StepLinkRequest) -> ApiResult<Option<RouteTarget>> {
    if request.ends_flow {
        return Ok(Some(RouteTarget::End));
    }
    Ok(optional_uuid("toStepId", request.to_step_id.as_deref())?.map(RouteTarget::Step))
}

async fn remove(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.step_links.delete(uuid("id", &id)?).await?))
}

async fn reset(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.flow.reset(uuid("taskId", &task_id)?).await?))
}

async fn place(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<StepFlowPlaceRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let id = uuid("id", &id)?;
    let place = match (request.x, request.y) {
        (None, None) => None,
        (x, y) => {
            Validator::new().not_null("x", &x).not_null("y", &y).finish()?;
            Some(FlowPlace::new(x.unwrap_or_default(), y.unwrap_or_default())?)
        }
    };
    Ok(Json(state.steps.place_in_flow(id, place).await?))
}

async fn limit_visits(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<StepMaxVisitsRequest>,
) -> ApiResult<Json<Vec<TaskStepView>>> {
    let id = uuid("id", &id)?;
    let budget = request.max_visits.map(VisitBudget::new).transpose()?;
    Ok(Json(state.steps.limit_visits(id, budget).await?))
}

async fn clear_places(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.steps.clear_flow_places(uuid("taskId", &task_id)?).await?))
}
