use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::ws::Message;
use serde_json::json;
use tokio::sync::mpsc;

use crate::pty::Listener;

use super::outbound::Outbound;
use super::socket_controller::{NORMAL, SEND_BUFFER_LIMIT_BYTES, SESSION_NOT_RELIABLE};

/// The listener half: queues frames for the writer task, and gives up on a pane that has fallen
/// more than the buffer limit behind.
pub(super) struct SocketListener {
    pub(super) sender: mpsc::UnboundedSender<Outbound>,
    pub(super) pending: Arc<AtomicUsize>,
    pub(super) closed: Arc<AtomicBool>,
}

impl SocketListener {
    pub(super) fn queue(&self, outbound: Outbound, size: usize) {
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
