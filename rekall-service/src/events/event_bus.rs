use tokio::sync::broadcast;

use super::{DomainEvent, PublishedEvent};

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<PublishedEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(4096);
        Self { sender }
    }

    pub fn publish(&self, event: impl DomainEvent) {
        self.announce(std::sync::Arc::new(event));
    }

    /// Announce an event already behind its `Arc`, as a committed transaction holds it.
    pub fn announce(&self, event: PublishedEvent) {
        // No subscriber is not an error: nobody is watching yet.
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PublishedEvent> {
        self.sender.subscribe()
    }
}
