use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::response::sse::Event;
use futures::stream::{self, Stream, StreamExt};
use rekall_console::chat::ChatService;
use rekall_core::chat::ChatEvent;
use rekall_kernel::Id;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::sse_listener::{Frame, Queued, SseListener};

/// The two ends of one reader's queue.
pub(in crate::chat) struct EventFeed {
    pub(in crate::chat) listener: Arc<SseListener>,
    receiver: mpsc::UnboundedReceiver<Queued>,
    pending: Arc<AtomicUsize>,
}

impl EventFeed {
    pub(in crate::chat) fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let pending = Arc::new(AtomicUsize::new(0));
        Self { listener: Arc::new(SseListener::new(sender, pending.clone())), receiver, pending }
    }

    /// The backlog, then every event as it comes, until the session ends or the reader is dropped.
    /// The attachment is let go of when the stream is, whichever end closed it.
    pub(in crate::chat) fn into_stream(self, backlog: Vec<ChatEvent>, attachment: Attachment) -> impl Stream<Item = Result<Event, Infallible>> {
        let replayed = stream::iter(backlog.iter().filter_map(Frame::of).collect::<Vec<_>>());
        let pending = self.pending;
        let live = UnboundedReceiverStream::new(self.receiver)
            .take_while(|queued| std::future::ready(matches!(queued, Queued::Frame(_))))
            .filter_map(move |queued| {
                let frame = match queued {
                    Queued::Frame(frame) => {
                        pending.fetch_sub(frame.json.len(), Ordering::SeqCst);
                        Some(frame)
                    }
                    Queued::Close => None,
                };
                std::future::ready(frame)
            });
        replayed.chain(live).map(move |frame| {
            let _keep = &attachment;
            Ok(Event::default().id(frame.seq.to_string()).data(frame.json))
        })
    }
}

/// Detaches the reader from its session when dropped.
pub(in crate::chat) struct Attachment {
    pub(in crate::chat) chat: ChatService,
    pub(in crate::chat) session_id: Id,
    pub(in crate::chat) token: u64,
}

impl Drop for Attachment {
    fn drop(&mut self) {
        self.chat.detach(self.session_id, self.token);
    }
}
