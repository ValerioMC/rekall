use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use rekall_core::queue::RunQueueView;
use rekall_kernel::RekallError;
use rekall_web::error::ApiResult;
use rekall_web::extract::{uuid, JsonBody, OptionalJsonBody};

use super::{AddRequest, MoveRequest, SettingsRequest, StartRequest};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/run-queue", get(view))
        .route("/api/run-queue/settings", put(settings))
        .route("/api/run-queue/items", post(add))
        .route("/api/run-queue/items/{item_id}", delete(remove))
        .route("/api/run-queue/items/{item_id}/position", put(move_to))
        .route("/api/run-queue/clear", post(clear_settled))
        .route("/api/run-queue/start", post(start))
        .route("/api/run-queue/stop", post(stop))
}

async fn view(State(state): State<ApiState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.run_queue.view().await?))
}

async fn settings(State(state): State<ApiState>, JsonBody(request): JsonBody<SettingsRequest>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.run_queue.update_settings(request.into()).await?))
}

async fn add(State(state): State<ApiState>, JsonBody(request): JsonBody<AddRequest>) -> ApiResult<Json<RunQueueView>> {
    let Some(task_id) = request.task_id else {
        return Err(RekallError::illegal("Name the task to queue.").into());
    };
    Ok(Json(state.run_queue.add(task_id).await?))
}

async fn remove(State(state): State<ApiState>, Path(item_id): Path<String>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.run_queue.remove(uuid("itemId", &item_id)?).await?))
}

async fn move_to(
    State(state): State<ApiState>,
    Path(item_id): Path<String>,
    JsonBody(request): JsonBody<MoveRequest>,
) -> ApiResult<Json<RunQueueView>> {
    let item_id = uuid("itemId", &item_id)?;
    let Some(index) = request.index else {
        return Err(RekallError::illegal("Say where the task goes.").into());
    };
    Ok(Json(state.run_queue.move_to(item_id, index).await?))
}

async fn clear_settled(State(state): State<ApiState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.run_queue.clear_settled().await?))
}

async fn start(State(state): State<ApiState>, OptionalJsonBody(request): OptionalJsonBody<StartRequest>) -> ApiResult<Json<RunQueueView>> {
    let start_at = request.and_then(|r| r.start_at);
    Ok(Json(state.run_queue.start(start_at).await?))
}

async fn stop(State(state): State<ApiState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.run_queue.stop().await?))
}
