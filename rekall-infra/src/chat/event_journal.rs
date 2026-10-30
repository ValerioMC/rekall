use std::collections::VecDeque;
use std::sync::Arc;

use rekall_core::chat::{ChatBody, ChatEvent, ChatListener};

/// A chat session's events in the order they happened, numbered from 1, and the listeners that
/// follow them. The ring keeps the newest events within a byte budget, so a client that attaches
/// late or reconnects gets what it missed, the way a terminal pane gets its scrollback. Listeners
/// are called under the caller's lock and must not block.
pub(super) struct EventJournal {
    next_seq: u64,
    ring: VecDeque<(ChatEvent, usize)>,
    held_bytes: usize,
    budget_bytes: usize,
    listeners: Vec<(u64, Arc<dyn ChatListener>)>,
}

impl EventJournal {
    pub(super) fn new(budget_bytes: usize) -> Self {
        Self { next_seq: 1, ring: VecDeque::new(), held_bytes: 0, budget_bytes, listeners: Vec::new() }
    }

    /// Numbers `body`, remembers it and hands it to every listener.
    pub(super) fn publish(&mut self, body: ChatBody) -> ChatEvent {
        let event = ChatEvent { seq: self.next_seq, body };
        self.next_seq += 1;
        self.remember(&event);
        for (_, listener) in &self.listeners {
            listener.event(&event);
        }
        event
    }

    /// Subscribes `listener` and returns the remembered events after `after_seq`, with nothing
    /// published in between.
    pub(super) fn attach(&mut self, token: u64, listener: Arc<dyn ChatListener>, after_seq: u64) -> Vec<ChatEvent> {
        self.listeners.push((token, listener));
        self.ring.iter().filter(|(event, _)| event.seq > after_seq).map(|(event, _)| event.clone()).collect()
    }

    pub(super) fn detach(&mut self, token: u64) {
        self.listeners.retain(|(held, _)| *held != token);
    }

    pub(super) fn take_listeners(&mut self) -> Vec<Arc<dyn ChatListener>> {
        std::mem::take(&mut self.listeners).into_iter().map(|(_, listener)| listener).collect()
    }

    fn remember(&mut self, event: &ChatEvent) {
        let weight = serde_json::to_vec(event).expect("an event of plain values always serialises").len();
        self.ring.push_back((event.clone(), weight));
        self.held_bytes += weight;
        while self.held_bytes > self.budget_bytes && self.ring.len() > 1 {
            if let Some((_, evicted)) = self.ring.pop_front() {
                self.held_bytes -= evicted;
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/chat/event_journal_tests.rs"]
mod tests;
