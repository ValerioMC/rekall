use std::sync::Arc;

use async_trait::async_trait;
use rekall_core::claude::{SessionKind, TerminalEnded, TerminalHost, TerminalOpening, TerminalView};
use rekall_kernel::{Id, RekallError, Result};
use tokio::sync::{broadcast, Mutex};
use tracing::warn;

use super::EndedBus;

/// The terminal host and the chat host as the one `TerminalHost` the queue, the relay and the console
/// reach. An opening goes to the host of its kind, every other call to the host that holds the
/// session, and the cap on live sessions counts both kinds together.
pub struct SessionHosts {
    terminals: Arc<dyn TerminalHost>,
    chats: Arc<dyn TerminalHost>,
    max_sessions: usize,
    ended: Arc<EndedBus>,
    opening_gate: Mutex<()>,
}

impl SessionHosts {
    /// Starts forwarding both hosts' ends onto one bus, so it must run inside the runtime.
    pub fn start(terminals: Arc<dyn TerminalHost>, chats: Arc<dyn TerminalHost>, max_sessions: usize) -> Self {
        let ended = Arc::new(EndedBus::new());
        forward_ends(terminals.subscribe_ended(), ended.clone());
        forward_ends(chats.subscribe_ended(), ended.clone());
        Self { terminals, chats, max_sessions, ended, opening_gate: Mutex::new(()) }
    }

    fn host_of_kind(&self, kind: SessionKind) -> &Arc<dyn TerminalHost> {
        match kind {
            SessionKind::Terminal => &self.terminals,
            SessionKind::Chat => &self.chats,
        }
    }

    fn host_holding(&self, id: Id) -> Option<&Arc<dyn TerminalHost>> {
        [&self.terminals, &self.chats].into_iter().find(|host| host.get(id).is_some())
    }

    fn resolve_kind(&self, opening: &TerminalOpening) -> Result<SessionKind> {
        let live: Vec<_> = self.list().into_iter().filter(|view| view.live).collect();
        if let Some(existing) = live.iter().find(|view| view.target() == opening.target) {
            return opening.kind_for_live_session(existing);
        }
        if live.len() < self.max_sessions {
            return Ok(opening.kind);
        }
        Err(RekallError::conflict(format!(
            "Rekall is already running {} terminals. Close one before opening another.",
            self.max_sessions
        )))
    }
}

fn forward_ends(mut ends: broadcast::Receiver<TerminalEnded>, bus: Arc<EndedBus>) {
    tokio::spawn(async move {
        loop {
            match ends.recv().await {
                Ok(ended) => bus.announce(ended.terminal_id, ended.task_id),
                Err(broadcast::error::RecvError::Lagged(skipped)) => warn!(skipped, "Session ends were missed by the merged bus"),
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

#[async_trait]
impl TerminalHost for SessionHosts {
    async fn open(&self, mut opening: TerminalOpening) -> Result<TerminalView> {
        let _gate = self.opening_gate.lock().await;
        opening.kind = self.resolve_kind(&opening)?;
        self.host_of_kind(opening.kind).open(opening).await
    }

    fn list(&self) -> Vec<TerminalView> {
        let mut views = self.terminals.list();
        views.extend(self.chats.list());
        views.sort_by_key(|view| view.started_at);
        views
    }

    fn get(&self, id: Id) -> Option<TerminalView> {
        self.terminals.get(id).or_else(|| self.chats.get(id))
    }

    async fn close(&self, id: Id, reason: &str) -> Option<TerminalView> {
        self.host_holding(id)?.close(id, reason).await
    }

    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        self.ended.subscribe()
    }

    fn has_live_for(&self, task_id: Id) -> bool {
        self.terminals.has_live_for(task_id) || self.chats.has_live_for(task_id)
    }

    fn work_session_for(&self, task_id: Id) -> Option<TerminalView> {
        self.terminals.work_session_for(task_id).or_else(|| self.chats.work_session_for(task_id))
    }

    async fn renew(&self, id: Id) -> Result<TerminalView> {
        let _gate = self.opening_gate.lock().await;
        match self.host_holding(id) {
            Some(host) => host.renew(id).await,
            None => Err(RekallError::conflict(format!("No session {id} is live to renew."))),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/session/session_hosts_tests.rs"]
mod tests;
