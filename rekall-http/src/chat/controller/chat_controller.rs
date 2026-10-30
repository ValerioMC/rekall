use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::post;
use axum::Router;

use crate::chat::request::answer_permission_request::AnswerPermissionRequest;
use crate::chat::request::send_message_request::SendMessageRequest;
use crate::shared::extract::{uuid, JsonBody};
use crate::shared::{no_content, ApiResult};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/chat/{id}/messages", post(send))
        .route("/api/chat/{id}/interrupt", post(interrupt))
}

pub fn permission_routes() -> Router<ApiState> {
    Router::new().route("/api/chat/{id}/permissions/{request_id}", post(answer_permission))
}

async fn send(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<SendMessageRequest>) -> ApiResult<Response> {
    let text = request.trimmed_text()?;
    state.chat.send(uuid("id", &id)?, &text)?;
    Ok(no_content())
}

async fn interrupt(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.chat.interrupt(uuid("id", &id)?)?;
    Ok(no_content())
}

async fn answer_permission(
    State(state): State<ApiState>,
    Path((id, request_id)): Path<(String, String)>,
    JsonBody(request): JsonBody<AnswerPermissionRequest>,
) -> ApiResult<Response> {
    let allow = request.allowed()?;
    state.chat.answer_permission(uuid("id", &id)?, &request_id, allow)?;
    Ok(no_content())
}
