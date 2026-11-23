use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};

use crate::decision::{AddDecisionRequest, DecisionResponse, EditDecisionRequest, TaskDecisionsResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody, Validator};
use crate::shared::{created, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{id}/decisions", get(list).post(add))
        .route("/api/decisions/{id}", put(edit).delete(remove))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskDecisionsResponse>> {
    Ok(Json(state.task_decisions.list_of(uuid("id", &id)?).await?.into()))
}

async fn add(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<AddDecisionRequest>) -> ApiResult<Response> {
    let task_id = uuid("id", &id)?;
    Validator::new().not_blank("question", request.question.as_deref()).not_blank("answer", request.answer.as_deref()).finish()?;
    let added = state.task_decisions.add(task_id, request.into_command()).await?;
    Ok(created(DecisionResponse::from(added)))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<EditDecisionRequest>) -> ApiResult<Json<DecisionResponse>> {
    Ok(Json(state.task_decisions.edit(uuid("id", &id)?, request.into_command()).await?.into()))
}

async fn remove(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.task_decisions.remove(uuid("id", &id)?).await?;
    Ok(no_content())
}
