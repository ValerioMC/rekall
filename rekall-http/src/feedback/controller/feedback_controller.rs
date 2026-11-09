use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, patch, post};
use axum::{Json, Router};

use crate::feedback::{FeedbackResponse, RetargetFeedbackRequest};
use crate::shared::error::ApiResult;
use crate::shared::extract::{present, uuid, JsonBody, Validator};
use crate::shared::{created, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{id}/feedback", get(list))
        .route("/api/tasks/{id}/feedback/passes", post(start_pass))
        .route("/api/feedback/{id}", patch(retarget))
        .route("/api/feedback/{id}/discard", post(discard))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<FeedbackResponse>>> {
    let views = state.feedback.list(uuid("id", &id)?).await?;
    Ok(Json(views.into_iter().map(FeedbackResponse::from).collect()))
}

async fn start_pass(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    let taken = state.feedback.start_pass(uuid("id", &id)?).await?;
    Ok(created(taken.into_iter().map(FeedbackResponse::from).collect::<Vec<_>>()))
}

async fn retarget(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<RetargetFeedbackRequest>) -> ApiResult<Json<FeedbackResponse>> {
    let feedback_id = uuid("id", &id)?;
    Validator::new().not_null("kind", &request.kind).finish()?;
    let kind = present("kind", request.kind)?;
    Ok(Json(state.feedback.retarget(feedback_id, kind).await?.into()))
}

async fn discard(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.feedback.discard_revision(uuid("id", &id)?).await?;
    Ok(no_content())
}
