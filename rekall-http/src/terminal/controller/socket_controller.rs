use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, LazyLock};

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use regex::Regex;
use rekall_kernel::Id;
use serde_json::json;
use tokio::sync::mpsc;

use crate::shared::is_allowed_origin;
use crate::terminal::support::socket_listener::SocketListener;
use crate::terminal::support::socket_reader::read_until_closed;
use crate::terminal::support::socket_writer::spawn_writer;
use crate::ApiState;

const MAX_BINARY_FRAME_BYTES: usize = 524_288;
pub(in crate::terminal) const SEND_BUFFER_LIMIT_BYTES: usize = 1_048_576;

/// The RFC 6455 close codes the terminal socket sends.
pub(in crate::terminal) const NORMAL: u16 = 1000;
const NOT_ACCEPTABLE: u16 = 1003;
const BAD_DATA: u16 = 1007;
pub(in crate::terminal) const TOO_BIG: u16 = 1009;
pub(in crate::terminal) const SERVER_ERROR: u16 = 1011;
pub(in crate::terminal) const SESSION_NOT_RELIABLE: u16 = 4500;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/terminal/{id}/io", get(upgrade))
}

async fn upgrade(
    State(state): State<ApiState>,
    Path(raw): Path<String>,
    headers: HeaderMap,
    socket: WebSocketUpgrade,
) -> Response {
    if let Some(origin) = headers.get("origin").and_then(|o| o.to_str().ok()) {
        if !is_allowed_origin(origin) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    socket
        .max_message_size(MAX_BINARY_FRAME_BYTES)
        .max_frame_size(MAX_BINARY_FRAME_BYTES)
        .on_upgrade(move |socket| serve(state, raw, socket))
        .into_response()
}

fn terminal_id_of(raw: &str) -> Option<Id> {
    static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-fA-F-]{36}$").expect("a valid pattern"));
    if !ID.is_match(raw) {
        return None;
    }
    raw.parse().ok()
}

async fn close_with(socket: &mut WebSocket, code: u16, reason: &str) {
    let _ = socket
        .send(Message::Close(Some(CloseFrame { code, reason: truncate_reason(reason).into() })))
        .await;
}

/// A close reason travels in a control frame, which holds 123 bytes of it at most.
pub(in crate::terminal) fn truncate_reason(reason: &str) -> String {
    let mut out = String::new();
    for c in reason.chars() {
        if out.len() + c.len_utf8() > 123 {
            break;
        }
        out.push(c);
    }
    out
}

async fn serve(state: ApiState, raw: String, mut socket: WebSocket) {
    let Some(terminal_id) = terminal_id_of(&raw) else {
        close_with(&mut socket, BAD_DATA, "Malformed terminal path.").await;
        return;
    };

    let (sender, outbound) = mpsc::unbounded_channel();
    let pending = Arc::new(AtomicUsize::new(0));
    let closed = Arc::new(AtomicBool::new(false));
    let listener = Arc::new(SocketListener { sender: sender.clone(), pending: pending.clone(), closed: closed.clone() });

    let (view, token) = match state.terminal_pipe.attach(terminal_id, listener.clone()) {
        Ok(attached) => attached,
        Err(gone) => {
            close_with(&mut socket, NOT_ACCEPTABLE, gone.message()).await;
            return;
        }
    };

    let ready = json!({
        "type": "ready",
        "id": view.id.to_string(),
        "anchors": view.anchors,
        "workingDir": view.working_dir,
    })
    .to_string();
    let (mut sink, stream) = socket.split();
    if sink.send(Message::Text(ready.into())).await.is_err() {
        state.terminal_pipe.detach(terminal_id, token);
        return;
    }

    let writer = spawn_writer(sink, outbound, pending);
    read_until_closed(&state, terminal_id, stream, &sender, &closed, &writer).await;
    state.terminal_pipe.detach(terminal_id, token);
    drop(sender);
    drop(listener);
    let _ = writer.await;
}

#[cfg(test)]
#[path = "../../../tests/unit/terminal/controller/socket_controller_tests.rs"]
mod tests;
