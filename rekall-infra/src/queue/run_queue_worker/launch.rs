//! Starting an item: the checks before a terminal opens on its task, and the permission wait noted
//! on an item that runs.

use rekall_kernel::{RekallError, Result};
use rekall_store::{RunQueueItemState, SessionProvider};
use rekall_core::claude::{Remaining, SessionKind, TaskWork, TerminalMode, TerminalOpening, TerminalTarget, TerminalView};
use rekall_core::queue::{RunQueueItemView, Snapshot};
use tracing::{info, warn};

use super::{Active, Worker};

impl Worker {
    pub(super) async fn launch(&mut self, item: RunQueueItemView, opening: TerminalOpening, parallel_sessions: i32) -> Result<()> {
        let before = match self.work.read(item.task_id).await {
            Ok(before) => before,
            Err(RekallError::UnknownAnchor(_)) => {
                return self.queue.mark_item(item.id, RunQueueItemState::Failed, Some("The task no longer exists.")).await;
            }
            Err(other) => return Err(other),
        };
        if !before.has_work() {
            return self.queue.mark_item(item.id, RunQueueItemState::Skipped, Some(nothing_open(&before))).await;
        }
        if let Some(existing) = self.terminals.list().into_iter().find(|session| session.live && session.task_id == Some(item.task_id)) {
            let reason = live_session_conflict(&existing);
            return self.queue.mark_item(item.id, RunQueueItemState::Failed, Some(&reason)).await;
        }
        if self.works_on_its_own_branch(&item, parallel_sessions).await {
            if let Err(refused) = self.worktrees.ensure(item.task_id).await {
                return self.park(&item, &refused).await;
            }
        }
        let opened = match self.terminals.open(opening).await {
            Ok(opened) => opened,
            Err(refused @ (RekallError::Conflict(_) | RekallError::IllegalArgument(_) | RekallError::UnknownAnchor(_))) => {
                return self.queue.mark_item(item.id, RunQueueItemState::Failed, Some(refused.message())).await;
            }
            Err(other) => return Err(other),
        };
        self.generation += 1;
        self.active.push(Active {
            generation: self.generation,
            item_id: item.id,
            task_id: item.task_id,
            terminal_id: opened.id,
            provider: opened.provider,
            model: opened.model,
            last_settled_at: before.last_settled_at,
            waiting_for_permission: false,
            closing: false,
        });
        self.queue.mark_item(item.id, RunQueueItemState::Running, None).await?;
        info!(anchor = %item.anchor, terminal_id = %opened.id, provider = opened.provider.label(), "Run queue: running");
        Ok(())
    }

    /// Says on the item in hand that its chat session waits for a person to answer a permission
    /// prompt, and takes it back once it is answered. The item keeps running meanwhile.
    pub(super) async fn note_permission_wait(&mut self, generation: u64) -> Result<()> {
        let Some(active) = self.active.iter_mut().find(|active| active.generation == generation) else { return Ok(()) };
        let waiting = self.terminals.get(active.terminal_id).is_some_and(|session| session.awaiting_permission);
        if waiting == active.waiting_for_permission {
            return Ok(());
        }
        active.waiting_for_permission = waiting;
        let item_id = active.item_id;
        self.queue.mark_item(item_id, RunQueueItemState::Running, waiting.then_some(WAITING_FOR_PERMISSION)).await
    }

    /// Two sessions at once need a folder each, and a project with a task manager keeps every task on a branch of its own.
    pub(super) async fn works_on_its_own_branch(&self, item: &RunQueueItemView, parallel_sessions: i32) -> bool {
        if parallel_sessions > 1 {
            return true;
        }
        self.worktrees.requires_branch(item.task_id).await.unwrap_or_else(|failure| {
            warn!(%failure, anchor = %item.anchor, "Run queue: could not tell whether the project keeps its tasks on branches");
            false
        })
    }

    /// An item that cannot be started now waits where it is, with the reason, and the queue goes
    /// on with the others. It is not offered again until the queue is armed anew.
    pub(super) async fn park(&mut self, item: &RunQueueItemView, refusal: &RekallError) -> Result<()> {
        warn!(%refusal, anchor = %item.anchor, "Run queue: could not give the task a worktree");
        let reason = format!("Could not make a worktree: {}", refusal.message());
        self.parked.insert(item.id);
        self.queue.mark_item(item.id, RunQueueItemState::Queued, Some(&reason)).await
    }
}

pub(super) fn queue_opening(item: &RunQueueItemView, snapshot: &Snapshot) -> TerminalOpening {
    let provider = item.launch.provider;
    let (skip_permissions, model, effort) = match provider {
        SessionProvider::Claude => {
            let (model, effort) = match &item.launch.claude_options {
                Some(options) => (options.model.clone(), options.effort.clone()),
                None => (snapshot.model.clone(), snapshot.effort.clone()),
            };
            (snapshot.skip_permissions, model, effort)
        }
        SessionProvider::Codex => {
            let options = item.launch.codex_options.clone().unwrap_or_default();
            (options.skip_permissions, options.model, options.effort)
        }
    };
    TerminalOpening {
        target: TerminalTarget::Task(item.task_id),
        step_id: None,
        provider,
        skip_permissions,
        model,
        effort,
        mode: TerminalMode::Work,
        kind: snapshot.session_kind,
    }
}

fn live_session_conflict(session: &TerminalView) -> String {
    let kind = match session.kind {
        SessionKind::Terminal => "Terminal",
        SessionKind::Chat => "Chat",
    };
    format!("{} {kind} is already open on this task. Close it and queue the task again.", session.provider.label())
}

fn nothing_open(work: &TaskWork) -> &'static str {
    if work.flow_ended {
        "Nothing to run: the flow already reached its end."
    } else if work.remaining == Remaining::Accepted {
        "Nothing to run: all of it is already accepted."
    } else {
        "Nothing to run: all of it is claimed and waiting for review."
    }
}

const WAITING_FOR_PERMISSION: &str = "Waiting for you to answer a permission prompt in the chat session.";
