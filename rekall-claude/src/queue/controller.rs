//! The console's handle on the run queue. Every call answers with the whole queue as it stands,
//! the same [`RunQueueView`] the `run-queue` SSE event carries, so the console has one shape to
//! draw from whichever arrives first.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use rekall_api::error::ApiResult;
use rekall_api::extract::{uuid, JsonBody, OptionalJsonBody};
use rekall_common::{Id, Instant, RekallError};
use serde::Deserialize;

use super::RunQueueView;
use crate::ClaudeState;

pub fn routes() -> Router<ClaudeState> {
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

/// `ceilingPercent` null is no ceiling; `model` and `effort` blank are the account's.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsRequest {
    ceiling_percent: Option<i32>,
    skip_permissions: Option<bool>,
    model: Option<String>,
    effort: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddRequest {
    task_id: Option<Id>,
}

#[derive(Deserialize)]
struct MoveRequest {
    index: Option<i32>,
}

/// `startAt` null starts at once.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartRequest {
    start_at: Option<Instant>,
}

async fn view(State(state): State<ClaudeState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.queue.view().await?))
}

async fn settings(State(state): State<ClaudeState>, JsonBody(request): JsonBody<SettingsRequest>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(
        state
            .queue
            .update_settings(
                request.ceiling_percent,
                request.skip_permissions == Some(true),
                request.model.as_deref(),
                request.effort.as_deref(),
            )
            .await?,
    ))
}

async fn add(State(state): State<ClaudeState>, JsonBody(request): JsonBody<AddRequest>) -> ApiResult<Json<RunQueueView>> {
    let Some(task_id) = request.task_id else {
        return Err(RekallError::illegal("Name the task to queue.").into());
    };
    let added = state.queue.add(task_id).await?;
    state.runner.nudge();
    Ok(Json(added))
}

async fn remove(State(state): State<ClaudeState>, Path(item_id): Path<String>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.queue.remove(uuid("itemId", &item_id)?).await?))
}

async fn move_to(
    State(state): State<ClaudeState>,
    Path(item_id): Path<String>,
    JsonBody(request): JsonBody<MoveRequest>,
) -> ApiResult<Json<RunQueueView>> {
    let item_id = uuid("itemId", &item_id)?;
    let Some(index) = request.index else {
        return Err(RekallError::illegal("Say where the task goes.").into());
    };
    Ok(Json(state.queue.move_to(item_id, index).await?))
}

async fn clear_settled(State(state): State<ClaudeState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.queue.clear_settled().await?))
}

async fn start(State(state): State<ClaudeState>, OptionalJsonBody(request): OptionalJsonBody<StartRequest>) -> ApiResult<Json<RunQueueView>> {
    let start_at = request.and_then(|r| r.start_at);
    Ok(Json(state.runner.start_queue(&state.queue, start_at).await?))
}

async fn stop(State(state): State<ClaudeState>) -> ApiResult<Json<RunQueueView>> {
    Ok(Json(state.runner.stop().await?))
}
