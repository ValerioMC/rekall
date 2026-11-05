use rekall_core::claude::{Remaining, TaskWorkService};
use rekall_core::review::TaskReviewEvent;
use rekall_core::step::StepStreamEvent;
use rekall_core::worktree::{Merge, Release, TaskWorktreeService};
use rekall_core::{EventBus, PublishedEvent};
use rekall_kernel::Id;
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

/// Merges a task's branch into the project and then takes its worktree away, once all of its work
/// is accepted. A branch that cannot be merged, or a worktree with unsaved changes, is left alone
/// and logged. The branch with its commits always stays.
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
        Ok(state) if state.remaining == Remaining::Accepted => {
            if merged_into_project(worktrees, task_id).await {
                release_worktree(worktrees, task_id).await;
            }
        }
        Ok(_) => {}
        Err(failure) => warn!(%failure, %task_id, "Could not read a task to see whether its worktree can go"),
    }
}

async fn merged_into_project(worktrees: &TaskWorktreeService, task_id: Id) -> bool {
    match worktrees.merge_accepted(task_id).await {
        Ok(Merge::Merged) => {
            info!(%task_id, "Accepted task's branch merged into the project");
            true
        }
        Ok(Merge::AlreadyMerged | Merge::NothingToMerge) => true,
        Ok(Merge::KeptWithChanges) => {
            info!(%task_id, "Accepted task's branch is not merged: its worktree has unsaved changes");
            false
        }
        Ok(Merge::Refused(reason)) => {
            warn!(%reason, %task_id, "Accepted task's branch was not merged; the worktree stays");
            false
        }
        Err(failure) => {
            warn!(%failure, %task_id, "Could not merge the branch of an accepted task");
            false
        }
    }
}

async fn release_worktree(worktrees: &TaskWorktreeService, task_id: Id) {
    match worktrees.release(task_id).await {
        Ok(Release::KeptWithChanges) => info!(%task_id, "Accepted task keeps its worktree: it has unsaved changes"),
        Ok(Release::Removed) => info!(%task_id, "Accepted task's worktree removed"),
        Ok(Release::NoWorktree) => {}
        Err(failure) => warn!(%failure, %task_id, "Could not remove the worktree of an accepted task"),
    }
}

fn task_moved_by(event: &PublishedEvent) -> Option<Id> {
    if let Some(steps) = event.downcast::<StepStreamEvent>() {
        return Some(steps.task_id);
    }
    event.downcast::<TaskReviewEvent>().map(|review| review.task_id)
}
