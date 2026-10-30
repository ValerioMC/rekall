use rekall_core::claude::TerminalEnded;
use rekall_kernel::Id;
use tokio::sync::broadcast;

const CAPACITY: usize = 256;

/// Announces every end of a session to whoever listens, the run queue among them.
pub struct EndedBus {
    sender: broadcast::Sender<TerminalEnded>,
}

impl EndedBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CAPACITY);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TerminalEnded> {
        self.sender.subscribe()
    }

    pub fn announce(&self, terminal_id: Id, task_id: Option<Id>) {
        let _ = self.sender.send(TerminalEnded { terminal_id, task_id });
    }
}

impl Default for EndedBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/session/ended_bus_tests.rs"]
mod tests;
