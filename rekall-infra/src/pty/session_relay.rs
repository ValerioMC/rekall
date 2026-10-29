//! Moves a terminal on to a fresh `claude` session when the step it claimed was the last of its
//! group and the next step opens a session of its own.
//!
//! It watches the steps of every task that has a work terminal open, waits the settle grace so the
//! session can print its closing lines, and renews the terminal. A task the run queue is running
//! is left to the queue, which decides at the same boundary after its usage ceiling.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_kernel::Id;
use rekall_core::claude::{TaskWorkService, TerminalView};
use rekall_core::queue::RunQueueService;
use rekall_core::review::TaskReviewEvent;
use rekall_core::step::StepStreamEvent;
use rekall_core::{EventBus, PublishedEvent};
use rekall_store::{RunQueueItemState, RunQueueState};
use tokio::sync::broadcast::{self, error::RecvError};
use tracing::{info, warn};

use super::PtyTerminalManager;

pub struct SessionRelay {
    terminals: Arc<PtyTerminalManager>,
    work: TaskWorkService,
    queue: RunQueueService,
    settle_grace: Duration,
    waiting: Mutex<HashSet<Id>>,
}

impl SessionRelay {
    pub fn start(
        terminals: Arc<PtyTerminalManager>,
        work: TaskWorkService,
        queue: RunQueueService,
        events: &EventBus,
        settle_grace: Duration,
    ) {
        let relay = Arc::new(Self { terminals, work, queue, settle_grace, waiting: Mutex::new(HashSet::new()) });
        tokio::spawn(relay.follow(events.subscribe()));
    }

    async fn follow(self: Arc<Self>, mut bus: broadcast::Receiver<PublishedEvent>) {
        loop {
            match bus.recv().await {
                Ok(event) => {
                    if let Some(task_id) = task_moved_by(&event) {
                        tokio::spawn(self.clone().check(task_id));
                    }
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            }
        }
    }

    async fn check(self: Arc<Self>, task_id: Id) {
        let Some(terminal) = self.terminals.work_terminal_for(task_id) else { return };
        if !self.is_spent(&terminal).await || self.queue_runs(task_id).await {
            return;
        }
        if !self.waiting.lock().expect("never poisoned").insert(terminal.id) {
            return;
        }
        tokio::time::sleep(self.settle_grace).await;
        self.renew(task_id, terminal.id).await;
        self.waiting.lock().expect("never poisoned").remove(&terminal.id);
    }

    async fn renew(&self, task_id: Id, terminal_id: Id) {
        let Some(terminal) = self.terminals.work_terminal_for(task_id).filter(|terminal| terminal.id == terminal_id) else {
            return;
        };
        if !self.is_spent(&terminal).await {
            return;
        }
        match self.terminals.renew_session(terminal_id).await {
            Ok(renewed) => info!(%task_id, from = %terminal_id, to = %renewed.id, "A new session opened on the next step"),
            Err(failure) => warn!(%failure, %task_id, terminal_id = %terminal_id, "Could not open the next session"),
        }
    }

    async fn is_spent(&self, terminal: &TerminalView) -> bool {
        let Some(task_id) = terminal.task_id else { return false };
        match self.work.read(task_id).await {
            Ok(work) => work.session_is_spent(&terminal.started_at),
            Err(failure) => {
                warn!(%failure, %task_id, "Could not read a task's work");
                false
            }
        }
    }

    async fn queue_runs(&self, task_id: Id) -> bool {
        match self.queue.view().await {
            Ok(view) => {
                view.state == RunQueueState::Running
                    && view.items.iter().any(|item| item.task_id == task_id && item.state == RunQueueItemState::Running)
            }
            Err(failure) => {
                warn!(%failure, %task_id, "Could not read the run queue");
                true
            }
        }
    }
}

fn task_moved_by(event: &PublishedEvent) -> Option<Id> {
    if let Some(steps) = event.downcast::<StepStreamEvent>() {
        return Some(steps.task_id);
    }
    event.downcast::<TaskReviewEvent>().map(|review| review.task_id)
}
