//! The byte pipe for one open terminal pane, Rekall's one WebSocket. Binary frames are stdin,
//! `{"resize":[cols,rows]}` text frames set the window size, `{"in":"<base64>"}` is stdin too,
//! PTY output returns as binary frames, and a `{"type":"ended"}` frame is sent on exit.
//!
//! A browser does not apply CORS to a WebSocket, so the handshake is only accepted from a
//! loopback origin (or from a client that sends none). Outbound frames are queued with a cap: a
//! pane that stops reading is dropped rather than left to back up memory.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use regex::Regex;
use rekall_common::{Id, RekallError};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tracing::debug;

use crate::pty::Listener;
use crate::ClaudeState;

const MAX_BINARY_FRAME_BYTES: usize = 524_288;
const MAX_TEXT_FRAME_BYTES: usize = 65_536;
const SEND_BUFFER_LIMIT_BYTES: usize = 1_048_576;

/// RFC 6455 close codes Spring's `CloseStatus` named.
const NORMAL: u16 = 1000;
const NOT_ACCEPTABLE: u16 = 1003;
const BAD_DATA: u16 = 1007;
const TOO_BIG: u16 = 1009;
const SERVER_ERROR: u16 = 1011;
const SESSION_NOT_RELIABLE: u16 = 4500;

pub fn routes() -> Router<ClaudeState> {
    Router::new().route("/api/terminal/{id}/io", get(upgrade))
}

/// `setAllowedOriginPatterns`: `http://localhost`, `http://127.0.0.1` and `http://[::1]`, with
/// or without a port.
pub fn is_allowed_origin(origin: &str) -> bool {
    static LOOPBACK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^http://(localhost|127\.0\.0\.1|\[::1\])(:[0-9]*)?$").unwrap());
    LOOPBACK.is_match(&origin.to_lowercase())
}

async fn upgrade(
    State(state): State<ClaudeState>,
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

enum Outbound {
    Frame(Message),
    Close(u16, String),
}

/// The listener half: queues frames for the writer task, and gives up on a pane that has fallen
/// more than the buffer limit behind.
struct SocketListener {
    sender: mpsc::UnboundedSender<Outbound>,
    pending: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
}

impl SocketListener {
    fn queue(&self, outbound: Outbound, size: usize) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        if self.pending.fetch_add(size, Ordering::SeqCst) + size > SEND_BUFFER_LIMIT_BYTES {
            self.closed.store(true, Ordering::SeqCst);
            let _ = self.sender.send(Outbound::Close(SESSION_NOT_RELIABLE, String::new()));
            return;
        }
        let _ = self.sender.send(outbound);
    }
}

impl Listener for SocketListener {
    fn output(&self, data: &[u8]) {
        self.queue(Outbound::Frame(Message::Binary(data.to_vec().into())), data.len());
    }

    fn ended(&self, exit_code: i32, detail: &str) {
        let frame = json!({ "type": "ended", "exitCode": exit_code, "detail": detail }).to_string();
        let size = frame.len();
        self.queue(Outbound::Frame(Message::Text(frame.into())), size);
        let _ = self.sender.send(Outbound::Close(NORMAL, String::new()));
        self.closed.store(true, Ordering::SeqCst);
    }
}

fn terminal_id_of(raw: &str) -> Option<Id> {
    static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-fA-F-]{36}$").unwrap());
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
fn truncate_reason(reason: &str) -> String {
    let mut out = String::new();
    for c in reason.chars() {
        if out.len() + c.len_utf8() > 123 {
            break;
        }
        out.push(c);
    }
    out
}

async fn serve(state: ClaudeState, raw: String, mut socket: WebSocket) {
    let Some(terminal_id) = terminal_id_of(&raw) else {
        close_with(&mut socket, BAD_DATA, "Malformed terminal path.").await;
        return;
    };

    let (sender, mut outbound) = mpsc::unbounded_channel();
    let pending = Arc::new(AtomicUsize::new(0));
    let closed = Arc::new(AtomicBool::new(false));
    let listener = Arc::new(SocketListener { sender: sender.clone(), pending: pending.clone(), closed: closed.clone() });

    let (view, token) = match state.terminals.attach(terminal_id, listener.clone()) {
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
    let (mut sink, mut stream) = socket.split();
    if sink.send(Message::Text(ready.into())).await.is_err() {
        state.terminals.detach(terminal_id, token);
        return;
    }

    let writer_pending = pending.clone();
    let writer = tokio::spawn(async move {
        while let Some(message) = outbound.recv().await {
            match message {
                Outbound::Frame(frame) => {
                    let size = match &frame {
                        Message::Binary(bytes) => bytes.len(),
                        Message::Text(text) => text.len(),
                        _ => 0,
                    };
                    let sent = sink.send(frame).await;
                    writer_pending.fetch_sub(size, Ordering::SeqCst);
                    if sent.is_err() {
                        break;
                    }
                }
                Outbound::Close(code, reason) => {
                    let _ = sink.send(Message::Close(Some(CloseFrame { code, reason: reason.into() }))).await;
                    break;
                }
            }
        }
    });

    while let Some(received) = stream.next().await {
        let message = match received {
            Ok(message) => message,
            Err(failure) => {
                debug!("Terminal socket {terminal_id} transport error: {failure}");
                let _ = sender.send(Outbound::Close(SERVER_ERROR, String::new()));
                break;
            }
        };
        match message {
            Message::Binary(bytes) => feed(&state, terminal_id, &bytes, &sender),
            Message::Text(text) => {
                if text.len() > MAX_TEXT_FRAME_BYTES {
                    let _ = sender.send(Outbound::Close(TOO_BIG, String::new()));
                    break;
                }
                let Ok(node) = serde_json::from_str::<Value>(&text) else { continue };
                if let Some(resize) = node.get("resize").and_then(Value::as_array).filter(|r| r.len() == 2) {
                    state.terminals.resize(terminal_id, as_int(&resize[0]), as_int(&resize[1]));
                    continue;
                }
                if let Some(input) = node.get("in").and_then(Value::as_str) {
                    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(input) {
                        feed(&state, terminal_id, &bytes, &sender);
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
        if closed.load(Ordering::SeqCst) && writer.is_finished() {
            break;
        }
    }
    state.terminals.detach(terminal_id, token);
    drop(sender);
    drop(listener);
    let _ = writer.await;
}

/// Jackson's `asInt()`: a number's integer part, a numeric string parsed, anything else 0.
fn as_int(value: &Value) -> i64 {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).unwrap_or(0),
        Value::String(s) => s.trim().parse().unwrap_or(0),
        Value::Bool(b) => i64::from(*b),
        _ => 0,
    }
}

fn feed(state: &ClaudeState, terminal_id: Id, bytes: &[u8], sender: &mpsc::UnboundedSender<Outbound>) {
    if bytes.is_empty() {
        return;
    }
    if let Err(RekallError::Conflict(reason)) = state.terminals.write(terminal_id, bytes) {
        let _ = sender.send(Outbound::Close(NORMAL, truncate_reason(&reason)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_loopback_page_may_open_the_pipe() {
        for allowed in ["http://localhost:5173", "http://127.0.0.1:47355", "http://[::1]:47355", "http://localhost"] {
            assert!(is_allowed_origin(allowed), "{allowed}");
        }
        for refused in ["https://evil.example", "http://localhost.evil.example", "https://localhost:47355"] {
            assert!(!is_allowed_origin(refused), "{refused}");
        }
    }

    #[test]
    fn a_path_that_is_not_an_id_is_malformed() {
        assert!(terminal_id_of("not-an-id").is_none());
        assert!(terminal_id_of(&Id::random().to_string()).is_some());
    }
}
