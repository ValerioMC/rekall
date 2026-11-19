use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::terminal::support::outbound::Outbound;

/// Sends what the listener queued until a frame fails or a close is queued, counting each frame
/// off `pending` once it has gone out.
pub(in crate::terminal) fn spawn_writer(
    mut sink: SplitSink<WebSocket, Message>,
    mut outbound: mpsc::UnboundedReceiver<Outbound>,
    pending: Arc<AtomicUsize>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(message) = outbound.recv().await {
            match message {
                Outbound::Frame(frame) => {
                    let size = frame_size(&frame);
                    let sent = sink.send(frame).await;
                    pending.fetch_sub(size, Ordering::SeqCst);
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
    })
}

fn frame_size(frame: &Message) -> usize {
    match frame {
        Message::Binary(bytes) => bytes.len(),
        Message::Text(text) => text.len(),
        _ => 0,
    }
}
