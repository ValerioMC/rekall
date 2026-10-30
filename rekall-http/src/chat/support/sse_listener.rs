use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use rekall_core::chat::{ChatEvent, ChatListener};
use tokio::sync::mpsc;
use tracing::error;

/// How much serialised event a reader may be behind before it is dropped.
pub(in crate::chat) const SEND_BUFFER_LIMIT_BYTES: usize = 1_048_576;

/// One event as it travels to the reader: its `seq`, which is the frame's id, and its JSON.
pub(in crate::chat) struct Frame {
    pub(in crate::chat) seq: u64,
    pub(in crate::chat) json: String,
}

impl Frame {
    /// `None` when the event cannot be serialised, which is logged: the reader goes on without it.
    pub(in crate::chat) fn of(event: &ChatEvent) -> Option<Self> {
        match serde_json::to_string(event) {
            Ok(json) => Some(Self { seq: event.seq, json }),
            Err(failure) => {
                error!(%failure, seq = event.seq, "A chat event could not be serialised");
                None
            }
        }
    }
}

pub(in crate::chat) enum Queued {
    Frame(Frame),
    Close,
}

/// The listener half: queues frames for the response stream, and gives up on a reader that has
/// fallen more than the buffer limit behind.
pub(in crate::chat) struct SseListener {
    sender: mpsc::UnboundedSender<Queued>,
    pending: Arc<AtomicUsize>,
    closed: AtomicBool,
}

impl SseListener {
    pub(in crate::chat) fn new(sender: mpsc::UnboundedSender<Queued>, pending: Arc<AtomicUsize>) -> Self {
        Self { sender, pending, closed: AtomicBool::new(false) }
    }

    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        let _ = self.sender.send(Queued::Close);
    }
}

impl ChatListener for SseListener {
    fn event(&self, event: &ChatEvent) {
        if self.closed.load(Ordering::SeqCst) {
            return;
        }
        let Some(frame) = Frame::of(event) else { return };
        let size = frame.json.len();
        if self.pending.fetch_add(size, Ordering::SeqCst) + size > SEND_BUFFER_LIMIT_BYTES {
            self.close();
            return;
        }
        let _ = self.sender.send(Queued::Frame(frame));
    }

    fn ended(&self, _exit_code: i32, _detail: &str) {
        self.close();
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/chat/support/sse_listener_tests.rs"]
mod tests;
