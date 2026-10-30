use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

use crate::chat::support::event_feed::{Attachment, EventFeed};
use crate::shared::extract::uuid;
use crate::shared::{is_allowed_origin, ApiResult};
use crate::ApiState;

const KEEP_ALIVE_EVERY: Duration = Duration::from_secs(15);

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/chat/{id}/events", get(events))
}

#[derive(Deserialize)]
struct EventsQuery {
    after: Option<u64>,
}

async fn events(State(state): State<ApiState>, Path(id): Path<String>, Query(query): Query<EventsQuery>, headers: HeaderMap) -> ApiResult<Response> {
    if !origin_is_allowed(&headers) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    let session_id = uuid("id", &id)?;
    let feed = EventFeed::new();
    let attached = state.chat.attach(session_id, feed.listener.clone(), resume_after(&headers, query.after))?;
    let attachment = Attachment { chat: state.chat.clone(), session_id, token: attached.token };
    let stream = feed.into_stream(attached.backlog, attachment);
    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(KEEP_ALIVE_EVERY).text("keep-alive")).into_response())
}

fn origin_is_allowed(headers: &HeaderMap) -> bool {
    match headers.get("origin").and_then(|origin| origin.to_str().ok()) {
        Some(origin) => is_allowed_origin(origin),
        None => true,
    }
}

/// What a reconnecting `EventSource` last saw wins over what the page asked for first.
fn resume_after(headers: &HeaderMap, after: Option<u64>) -> u64 {
    headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse().ok())
        .or(after)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/controller/chat_events_controller_tests.rs"]
mod tests;
