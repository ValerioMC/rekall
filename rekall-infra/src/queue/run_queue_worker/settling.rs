//! Ending an item: its outcome, a pause at the usage ceiling, a terminal that went away, and
//! closing the terminal with the steps it had marked running.

use rekall_kernel::{Id, Result};
use rekall_store::RunQueueItemState;
use rekall_core::claude::TaskWork;
use rekall_core::queue::Snapshot;
use tracing::{info, warn};

use super::review::finished;
use super::super::usage_ceiling::{self, Verdict};
use super::{Active, Worker};

impl Worker {
    /// The item is done with; close its terminal, record the outcome, and go on to the next.
    pub(super) async fn settle(&mut self, generation: u64, outcome: RunQueueItemState, reason: &str) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.forget(generation).expect("checked above");
        self.close_session(current.clone(), "The run queue moved on to the next task.").await;
        self.queue.mark_item(current.item_id, outcome, Some(reason)).await?;
        self.advance().await
    }

    /// At the ceiling on a step boundary: close the session, put the item back at the head, hold.
    pub(super) async fn pause(&mut self, generation: u64, verdict: Verdict) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.forget(generation).expect("checked above");
        self.close_session(current.clone(), "The run queue reached its usage ceiling.").await;
        self.queue
            .mark_item(current.item_id, RunQueueItemState::Queued, Some("Paused at the usage ceiling; it picks up at the next open step."))
            .await?;
        if self.active.is_empty() {
            self.queue.hold(verdict.resume_at, verdict.reason.as_deref()).await?;
        }
        info!(resume_at = ?verdict.resume_at, reason = ?verdict.reason, "Run queue: paused at the ceiling");
        Ok(())
    }

    /// The terminal in hand went away without the runner closing it.
    pub(super) async fn session_ended(&mut self, terminal_id: Id) -> Result<()> {
        let Some(current) = self.active.iter().find(|active| active.terminal_id == terminal_id).cloned() else { return Ok(()) };
        if current.closing {
            return Ok(());
        }
        self.forget(current.generation);
        self.release_running_steps(current.task_id).await;
        let after = self.read_or_none(current.task_id).await;
        match after {
            Some(after) if !after.has_work() => {
                self.queue.mark_item(current.item_id, RunQueueItemState::Finished, Some(finished(&after))).await?;
            }
            _ => {
                self.queue
                    .mark_item(current.item_id, RunQueueItemState::Failed, Some("The session ended before its work was claimed."))
                    .await?;
            }
        }
        self.advance().await
    }

    pub(super) async fn ceiling(&self, snapshot: &Snapshot) -> Verdict {
        if snapshot.ceiling_percent.is_none() {
            return Verdict::go();
        }
        let reading = self.usage.current().await;
        usage_ceiling::check(Some(&reading), snapshot.ceiling_percent, snapshot.model.as_deref(), (self.clock)())
    }

    /// Close the item's terminal and put back any step its session had marked running: the next
    /// session has to find it open to pick it up.
    pub(super) async fn close_session(&self, mut current: Active, reason: &str) {
        current.closing = true;
        self.terminals.close(current.terminal_id, reason).await;
        self.release_running_steps(current.task_id).await;
    }

    pub(super) async fn release_running_steps(&self, task_id: Id) {
        if let Some(state) = self.read_or_none(task_id).await {
            for step_id in state.running_step_ids {
                if let Err(failed) = self.steps.release_running(Some(step_id)).await {
                    warn!(%failed, %step_id, "Run queue: releasing a step failed");
                }
            }
        }
    }

    pub(super) async fn read_or_none(&self, task_id: Id) -> Option<TaskWork> {
        self.work.read(task_id).await.ok()
    }
}
