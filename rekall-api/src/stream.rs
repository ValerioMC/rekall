//! `GET /api/steps/stream`: the console's one Server-Sent Events feed. Every committed step,
//! review, wrapup and commit-reference change, and every run queue change, reaches every open
//! console under its frame name. A console that falls too far behind loses the frames it missed,
//! which is what a failed `SseEmitter.send` dropped too.

use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::response::sse::{Event, Sse};
use futures::stream::{self, Stream, StreamExt};
use rekall_service::EventBus;
use tokio::sync::watch;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;

pub struct StepEventStream {
    bus: EventBus,
    clients: Arc<Mutex<HashSet<u64>>>,
    next: AtomicU64,
    closing: watch::Sender<bool>,
}

/// Takes its feed off the list when the connection goes, whichever end closed it.
struct ClientGuard(Arc<Mutex<HashSet<u64>>>, u64);

impl Drop for ClientGuard {
    fn drop(&mut self) {
        if let Ok(mut clients) = self.0.lock() {
            clients.remove(&self.1);
        }
    }
}

impl StepEventStream {
    pub fn new(bus: EventBus) -> Self {
        let (closing, _) = watch::channel(false);
        Self { bus, clients: Arc::new(Mutex::new(HashSet::new())), next: AtomicU64::new(0), closing }
    }

    /// A new feed: an `open` frame saying `ready`, then every event as it commits.
    pub fn open(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>> + use<>> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut clients) = self.clients.lock() {
            clients.insert(id);
        }
        let guard = ClientGuard(self.clients.clone(), id);
        let mut closing = self.closing.subscribe();
        let events = BroadcastStream::new(self.bus.subscribe()).filter_map(|received| async move {
            match received {
                Ok(event) => Some(Event::default().event(event.name()).data(event.payload().to_string())),
                Err(BroadcastStreamRecvError::Lagged(_)) => None,
            }
        });
        let ended = async move {
            let _ = closing.wait_for(|closed| *closed).await;
        };
        let feed = stream::once(async { Event::default().event("open").data("ready") })
            .chain(events)
            .take_until(ended)
            .map(move |event| {
                let _keep = &guard;
                Ok(event)
            });
        Sse::new(feed)
    }

    /// Complete every open feed: graceful shutdown would otherwise wait on each of them.
    pub fn release_on_shutdown(&self) {
        self.closing.send_replace(true);
        if let Ok(mut clients) = self.clients.lock() {
            clients.clear();
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.lock().map(|c| c.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rekall_service::step::StepStreamEvent;
    use rekall_service::wrapup::WrapupStreamEvent;
    use rekall_service::DomainEvent;

    #[tokio::test]
    async fn each_open_connection_is_counted_then_dropped_when_the_app_closes() {
        let stream = StepEventStream::new(EventBus::new());
        let _a = stream.open();
        let _b = stream.open();
        let _c = stream.open();
        assert_eq!(stream.client_count(), 3);
        stream.release_on_shutdown();
        assert_eq!(stream.client_count(), 0, "graceful shutdown must find nothing holding a request open");
    }

    #[tokio::test]
    async fn a_change_after_shutdown_reaches_nobody_and_does_not_fail() {
        let bus = EventBus::new();
        let stream = StepEventStream::new(bus.clone());
        let _open = stream.open();
        stream.release_on_shutdown();
        bus.publish(DomainEvent::Steps(StepStreamEvent { task_id: rekall_common::Id::random(), steps: vec![] }));
        bus.publish(DomainEvent::Wrapup(WrapupStreamEvent { task_id: rekall_common::Id::random(), wrapup: None, deleted: true }));
        assert_eq!(stream.client_count(), 0);
    }

    #[tokio::test]
    async fn shutdown_with_no_console_connected_or_twice_is_harmless() {
        let stream = StepEventStream::new(EventBus::new());
        stream.release_on_shutdown();
        let _open = stream.open();
        stream.release_on_shutdown();
        stream.release_on_shutdown();
        assert_eq!(stream.client_count(), 0);
    }
}
