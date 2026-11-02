use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use rekall_kernel::{Id, Instant, RekallError, Result};
use tokio::sync::broadcast;

use crate::claude::{SessionKind, TerminalEnded, TerminalHost, TerminalMode, TerminalOpening, TerminalTarget, TerminalView};

const ENDED_CAPACITY: usize = 16;

/// The `TerminalHost` tests use: sessions held in memory, opened as the kind asked for, that keep a
/// record of every opening and renewal. A renewal replaces the session by another of its kind.
pub struct InMemoryTerminalHost {
    live: Mutex<Vec<TerminalView>>,
    openings: Mutex<Vec<TerminalOpening>>,
    renewed: Mutex<Vec<Id>>,
    ended: broadcast::Sender<TerminalEnded>,
}

impl Default for InMemoryTerminalHost {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTerminalHost {
    pub fn new() -> Self {
        let (ended, _) = broadcast::channel(ENDED_CAPACITY);
        Self { live: Mutex::new(Vec::new()), openings: Mutex::new(Vec::new()), renewed: Mutex::new(Vec::new()), ended }
    }

    /// A live session of `kind` on `target`, the way this host would open one.
    pub fn session_on(target: TerminalTarget, kind: SessionKind) -> TerminalView {
        let (project_id, task_id) = match target {
            TerminalTarget::Task(task_id) => (Id::random(), Some(task_id)),
            TerminalTarget::Project(project_id) => (project_id, None),
        };
        let now = Instant::now();
        TerminalView {
            id: Id::random(),
            project_id,
            task_id,
            step_id: None,
            anchors: "project:vega".into(),
            working_dir: "/work/vega".into(),
            project_label: "vega".into(),
            task_label: None,
            task_title: None,
            mode: TerminalMode::Work,
            skip_permissions: false,
            model: None,
            effort: None,
            kind,
            live: true,
            awaiting_permission: false,
            started_at: now,
            last_activity_at: now,
            connections: Vec::new(),
        }
    }

    pub fn add(&self, view: TerminalView) {
        self.sessions().push(view);
    }

    pub fn openings(&self) -> Vec<TerminalOpening> {
        self.openings.lock().expect("never poisoned").clone()
    }

    pub fn renewed(&self) -> Vec<Id> {
        self.renewed.lock().expect("never poisoned").clone()
    }

    /// Removes the session and announces its end, as a real host does when a session exits.
    pub fn end(&self, id: Id) {
        let mut sessions = self.sessions();
        if let Some(position) = sessions.iter().position(|view| view.id == id) {
            let view = sessions.remove(position);
            let _ = self.ended.send(TerminalEnded { terminal_id: view.id, task_id: view.task_id });
        }
    }

    /// Marks the session as stopped on a permission prompt, or as past it.
    pub fn awaiting_permission(&self, id: Id, waiting: bool) {
        if let Some(view) = self.sessions().iter_mut().find(|view| view.id == id) {
            view.awaiting_permission = waiting;
        }
    }

    fn sessions(&self) -> MutexGuard<'_, Vec<TerminalView>> {
        self.live.lock().expect("never poisoned")
    }
}

#[async_trait]
impl TerminalHost for InMemoryTerminalHost {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView> {
        let view = Self::session_on(opening.target, opening.kind);
        self.openings.lock().expect("never poisoned").push(opening);
        self.sessions().push(view.clone());
        Ok(view)
    }

    fn list(&self) -> Vec<TerminalView> {
        self.sessions().clone()
    }

    fn get(&self, id: Id) -> Option<TerminalView> {
        self.sessions().iter().find(|view| view.id == id).cloned()
    }

    async fn close(&self, id: Id, _reason: &str) -> Option<TerminalView> {
        let mut sessions = self.sessions();
        let position = sessions.iter().position(|view| view.id == id)?;
        Some(sessions.remove(position))
    }

    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        self.ended.subscribe()
    }

    fn has_live_for(&self, task_id: Id) -> bool {
        self.sessions().iter().any(|view| view.task_id == Some(task_id))
    }

    fn work_session_for(&self, task_id: Id) -> Option<TerminalView> {
        self.sessions().iter().find(|view| view.task_id == Some(task_id)).cloned()
    }

    async fn renew(&self, id: Id) -> Result<TerminalView> {
        let old = self.close(id, "renewed").await.ok_or_else(|| RekallError::conflict(format!("No session {id} to renew.")))?;
        self.renewed.lock().expect("never poisoned").push(id);
        let renewed = Self::session_on(old.target(), old.kind);
        self.sessions().push(renewed.clone());
        Ok(renewed)
    }
}
