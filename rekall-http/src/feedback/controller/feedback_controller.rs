use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_console::feedback::AddFeedbackCommand;
use rekall_store::task_feedback::BODY_MAX_CHARACTERS as MAX_BODY_CHARACTERS;

use crate::feedback::{AddFeedbackRequest, EditFeedbackRequest, FeedbackResponse};
use crate::shared::error::ApiResult;
use crate::shared::extract::{present, uuid, JsonBody, Validator};
use crate::shared::{created, no_content};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/tasks/{id}/feedback", get(list).post(add))
        .route("/api/tasks/{id}/feedback/passes", post(start_pass))
        .route("/api/feedback/{id}", patch(edit).delete(delete))
        .route("/api/feedback/{id}/discard", post(discard))
}

async fn list(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<Vec<FeedbackResponse>>> {
    let views = state.feedback.list(uuid("id", &id)?).await?;
    Ok(Json(views.into_iter().map(FeedbackResponse::from).collect()))
}

async fn add(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<AddFeedbackRequest>) -> ApiResult<Response> {
    let task_id = uuid("id", &id)?;
    Validator::new()
        .not_null("kind", &request.kind)
        .not_blank("bodyMarkdown", request.body_markdown.as_deref())
        .at_most("bodyMarkdown", request.body_markdown.as_deref(), MAX_BODY_CHARACTERS)
        .finish()?;
    let command = AddFeedbackCommand {
        kind: present("kind", request.kind)?,
        body: present("bodyMarkdown", request.body_markdown)?,
        follows_seq: request.follows_seq,
    };
    Ok(created(FeedbackResponse::from(state.feedback.add(task_id, command).await?)))
}

async fn start_pass(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    let taken = state.feedback.start_pass(uuid("id", &id)?).await?;
    Ok(created(taken.into_iter().map(FeedbackResponse::from).collect::<Vec<_>>()))
}

async fn edit(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<EditFeedbackRequest>) -> ApiResult<Json<FeedbackResponse>> {
    let feedback_id = uuid("id", &id)?;
    Validator::new()
        .not_blank("bodyMarkdown", request.body_markdown.as_deref())
        .at_most("bodyMarkdown", request.body_markdown.as_deref(), MAX_BODY_CHARACTERS)
        .finish()?;
    let body = present("bodyMarkdown", request.body_markdown)?;
    Ok(Json(state.feedback.edit(feedback_id, &body).await?.into()))
}

async fn delete(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.feedback.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

async fn discard(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.feedback.discard_revision(uuid("id", &id)?).await?;
    Ok(no_content())
}
