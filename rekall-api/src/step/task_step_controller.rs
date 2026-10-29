use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_core::step::TaskStepView;

use rekall_web::{created, no_content};
use crate::step::{TaskStepMoveRequest, TaskStepPatchRequest, TaskStepRequest};
use rekall_web::error::ApiResult;
use rekall_web::extract::{present, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/steps", get(list))
        .route("/api/tasks/{task_id}/steps", get(list_on).post(add))
        .route("/api/steps/{id}", patch(edit).delete(delete))
        .route("/api/steps/{id}/move", post(move_step))
        .route("/api/steps/{id}/send-back", post(send_back))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.steps.find_all().await?))
}

async fn list_on(State(state): State<ApiState>, Path(task_id): Path<String>) -> ApiResult<Json<Vec<TaskStepView>>> {
    Ok(Json(state.steps.find_by_task(uuid("taskId", &task_id)?).await?))
}

async fn add(State(state): State<ApiState>, Path(task_id): Path<String>, JsonBody(request): JsonBody<TaskStepRequest>) -> ApiResult<Response> {
    let task_id = uuid("taskId", &task_id)?;
    Validator::new().not_blank("title", request.title.as_deref()).finish()?;
    Ok(created(state.steps.add(task_id, request.title.as_deref(), request.body_markdown.as_deref()).await?))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<TaskStepPatchRequest>) -> ApiResult<Json<TaskStepView>> {
    let id = uuid("id", &id)?;
    Ok(Json(
        state
            .steps
            .edit(id, request.title.as_deref(), request.body_markdown.as_deref(), request.done, request.draft)
            .await?,
    ))
}

async fn send_back(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<TaskStepView>> {
    Ok(Json(state.steps.send_back(uuid("id", &id)?).await?))
}

async fn move_step(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<TaskStepMoveRequest>) -> ApiResult<Json<Vec<TaskStepView>>> {
    let id = uuid("id", &id)?;
    let position = present("position", request.position)?;
    Ok(Json(state.steps.move_to(id, position).await?))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.steps.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}
