//! Reviewing the task in hand when a step or its review line moves: finished, past the ceiling, or
//! a session that has spent its steps.

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::RunQueueItemState;
use rekall_core::claude::{Remaining, TaskWork};

use super::{Command, Worker};

impl Worker {
    /// A step or the review line moved on the task in hand: finished, or a boundary to check.
    pub(super) async fn review(&mut self, task_id: Id) -> Result<()> {
        let Some(current) = self.active.iter().find(|active| active.task_id == task_id).cloned() else { return Ok(()) };
        if current.closing {
            return Ok(());
        }
        let now = match self.work.read(task_id).await {
            Ok(now) => now,
            Err(RekallError::UnknownAnchor(_)) => {
                self.forget(current.generation);
                self.close_session(current, "The task was deleted.").await;
                return self.advance().await;
            }
            Err(other) => return Err(other),
        };
        if !now.has_work() {
            self.mark_closing(current.generation);
            self.later(Command::Settle {
                generation: current.generation,
                outcome: RunQueueItemState::Finished,
                reason: finished(&now).into(),
            });
            return Ok(());
        }
        if now.settled_steps > current.settled_steps {
            if let Some(active) = self.active_mut(current.generation) {
                active.settled_steps = now.settled_steps;
            }
            let snapshot = self.queue.snapshot().await?;
            let verdict = self.ceiling(&snapshot).await;
            if !verdict.clear {
                self.mark_closing(current.generation);
                self.later(Command::Pause { generation: current.generation, verdict });
                return Ok(());
            }
        }
        if self.is_spent(current.terminal_id, &now) {
            self.later(Command::Renew { generation: current.generation });
        }
        Ok(())
    }

    pub(super) fn is_spent(&self, terminal_id: Id, work: &TaskWork) -> bool {
        self.terminals.get(terminal_id).is_some_and(|terminal| work.session_is_spent(&terminal.started_at))
    }
}

pub(super) fn finished(work: &TaskWork) -> &'static str {
    if work.flow_ended {
        "Flow finished: the steps it did not need were left unrun."
    } else if work.remaining == Remaining::Accepted {
        "All of it accepted."
    } else {
        "All of it claimed and waiting for review."
    }
}
