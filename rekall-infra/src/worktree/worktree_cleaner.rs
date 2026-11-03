use rekall_core::claude::{Remaining, TaskWorkService};
use rekall_core::review::TaskReviewEvent;
use rekall_core::step::StepStreamEvent;
use rekall_core::worktree::{Release, TaskWorktreeService};
use rekall_core::{EventBus, PublishedEvent};
use rekall_kernel::Id;
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

/// Takes a task's worktree away once all of its work is accepted. A worktree with unsaved changes
/// is left alone and logged, and the branch with its commits always stays.
pub struct WorktreeCleaner;

impl WorktreeCleaner {
    /// Listens to every committed step and review move until the event bus closes.
    pub fn start(events: &EventBus, work: TaskWorkService, worktrees: TaskWorktreeService) {
        let mut bus = events.subscribe();
        tokio::spawn(async move {
            loop {
                match bus.recv().await {
                    Ok(event) => {
                        if let Some(task_id) = task_moved_by(&event) {
                            clean_if_accepted(&work, &worktrees, task_id).await;
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return,
                }
            }
        });
    }
}

async fn clean_if_accepted(work: &TaskWorkService, worktrees: &TaskWorktreeService, task_id: Id) {
    match work.read(task_id).await {
        Ok(state) if state.remaining == Remaining::Accepted => match worktrees.release(task_id).await {
            Ok(Release::KeptWithChanges) => info!(%task_id, "Accepted task keeps its worktree: it has unsaved changes"),
            Ok(Release::Removed) => info!(%task_id, "Accepted task's worktree removed"),
            Ok(Release::NoWorktree) => {}
            Err(failure) => warn!(%failure, %task_id, "Could not remove the worktree of an accepted task"),
        },
        Ok(_) => {}
        Err(failure) => warn!(%failure, %task_id, "Could not read a task to see whether its worktree can go"),
    }
}

fn task_moved_by(event: &PublishedEvent) -> Option<Id> {
    if let Some(steps) = event.downcast::<StepStreamEvent>() {
        return Some(steps.task_id);
    }
    event.downcast::<TaskReviewEvent>().map(|review| review.task_id)
}
