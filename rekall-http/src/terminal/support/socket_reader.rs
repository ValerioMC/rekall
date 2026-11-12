use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::extract::ws::{Message, WebSocket};
use base64::Engine;
use futures::stream::SplitStream;
use futures::StreamExt;
use rekall_kernel::{Id, RekallError};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::terminal::controller::socket_controller::{truncate_reason, NORMAL, SERVER_ERROR, TOO_BIG};
use crate::terminal::support::outbound::Outbound;
use crate::ApiState;

const MAX_TEXT_FRAME_BYTES: usize = 65_536;

/// Feeds the terminal what the browser sends until the socket closes, a frame is refused, or the
/// listener has closed and its writer is done. A refusal queues its close code for the writer.
pub(in crate::terminal) async fn read_until_closed(
    state: &ApiState,
    terminal_id: Id,
    mut stream: SplitStream<WebSocket>,
    sender: &mpsc::UnboundedSender<Outbound>,
    closed: &AtomicBool,
    writer: &JoinHandle<()>,
) {
    while let Some(received) = stream.next().await {
        let message = match received {
            Ok(message) => message,
            Err(failure) => {
                debug!(%failure, %terminal_id, "Terminal socket transport error");
                let _ = sender.send(Outbound::Close(SERVER_ERROR, String::new()));
                break;
            }
        };
        if handle_message(state, terminal_id, message, sender).is_break() {
            break;
        }
        if closed.load(Ordering::SeqCst) && writer.is_finished() {
            break;
        }
    }
}

fn handle_message(state: &ApiState, terminal_id: Id, message: Message, sender: &mpsc::UnboundedSender<Outbound>) -> ControlFlow<()> {
    match message {
        Message::Binary(bytes) => feed(state, terminal_id, &bytes, sender),
        Message::Text(text) => return handle_text(state, terminal_id, &text, sender),
        Message::Close(_) => return ControlFlow::Break(()),
        _ => {}
    }
    ControlFlow::Continue(())
}

fn handle_text(state: &ApiState, terminal_id: Id, text: &str, sender: &mpsc::UnboundedSender<Outbound>) -> ControlFlow<()> {
    if text.len() > MAX_TEXT_FRAME_BYTES {
        let _ = sender.send(Outbound::Close(TOO_BIG, String::new()));
        return ControlFlow::Break(());
    }
    let Ok(node) = serde_json::from_str::<Value>(text) else { return ControlFlow::Continue(()) };
    if let Some(resize) = node.get("resize").and_then(Value::as_array).filter(|r| r.len() == 2) {
        state.terminal_pipe.resize(terminal_id, as_int(&resize[0]), as_int(&resize[1]));
        return ControlFlow::Continue(());
    }
    if let Some(input) = node.get("in").and_then(Value::as_str) {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(input) {
            feed(state, terminal_id, &bytes, sender);
        }
    }
    ControlFlow::Continue(())
}

/// Lenient on purpose: a number's integer part, a numeric string parsed, anything else 0.
fn as_int(value: &Value) -> i64 {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).unwrap_or(0),
        Value::String(s) => s.trim().parse().unwrap_or(0),
        Value::Bool(b) => i64::from(*b),
        _ => 0,
    }
}

fn feed(state: &ApiState, terminal_id: Id, bytes: &[u8], sender: &mpsc::UnboundedSender<Outbound>) {
    if bytes.is_empty() {
        return;
    }
    if let Err(RekallError::Conflict(reason)) = state.terminal_pipe.write(terminal_id, bytes) {
        let _ = sender.send(Outbound::Close(NORMAL, truncate_reason(&reason)));
    }
}
