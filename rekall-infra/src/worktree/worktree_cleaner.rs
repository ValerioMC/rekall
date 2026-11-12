use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rekall_core::claude::{Remaining, SessionKind, TaskWorkService, TerminalHost, TerminalMode, TerminalOpening, TerminalTarget};
use rekall_core::review::TaskReviewEvent;
use rekall_core::step::StepStreamEvent;
use rekall_core::worktree::{Merge, PullRequestLanding, Release, TaskPullRequestService, TaskWorktreeService};
use rekall_core::{EventBus, PublishedEvent};
use rekall_kernel::Id;
use tokio::sync::broadcast::error::RecvError;
use tracing::{info, warn};

const MERGE_SESSION_CLOSED: &str = "The branch is merged.";

/// Opens the pull request of a task whose work is claimed, on a project with the pull request flow
/// on. Merges a task's branch into the project and then takes its worktree away, once all of its work
/// is accepted. A branch that conflicts gets a `merge` session that resolves it in the task's
/// worktree; the merge is retried on every tick until it goes through. A merged task is closed. Unsaved changes and a
/// merge git refuses are left alone and logged. The branch with its commits always stays.
pub struct WorktreeCleaner;

impl WorktreeCleaner {
    /// Listens to every committed step and review move until the event bus closes, and looks at
    /// the conflicts being resolved every `check_every`.
    pub fn start(events: &EventBus, work: TaskWorkService, worktrees: TaskWorktreeService, pull_requests: TaskPullRequestService, terminals: Arc<dyn TerminalHost>, check_every: Duration) {
        let mut bus = events.subscribe();
        let mut cleaner = Cleaner { work, worktrees, pull_requests, terminals, resolutions: HashMap::new() };
        tokio::spawn(async move {
            let mut clock = tokio::time::interval_at(tokio::time::Instant::now() + check_every, check_every);
            loop {
                tokio::select! {
                    received = bus.recv() => match received {
                        Ok(event) => {
                            if let Some(task_id) = task_moved_by(&event) {
                                cleaner.open_pull_request(task_id).await;
                                cleaner.clean_if_accepted(task_id).await;
                            }
                        }
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => return,
                    },
                    _ = clock.tick() => cleaner.follow_resolutions().await,
                }
            }
        });
    }
}

/// Where a task's conflict stands: a session is resolving it, or the session ended without the
/// branch merging and none is opened again until the task is accepted anew.
enum Resolution {
    Running(Id),
    Abandoned,
}

struct Cleaner {
    work: TaskWorkService,
    worktrees: TaskWorktreeService,
    pull_requests: TaskPullRequestService,
    terminals: Arc<dyn TerminalHost>,
    resolutions: HashMap<Id, Resolution>,
}

impl Cleaner {
    async fn open_pull_request(&self, task_id: Id) {
        if let Err(failure) = self.pull_requests.open_if_claimed(task_id).await {
            warn!(%failure, %task_id, "Could not look for the pull request of a claimed task");
        }
    }

    async fn clean_if_accepted(&mut self, task_id: Id) {
        match self.work.read(task_id).await {
            Ok(state) if state.remaining == Remaining::Accepted => self.bring_in(task_id).await,
            Ok(_) => {
                self.resolutions.remove(&task_id);
            }
            Err(failure) => warn!(%failure, %task_id, "Could not read a task to see whether its worktree can go"),
        }
    }

    async fn bring_in(&mut self, task_id: Id) {
        match self.pull_requests.merge_accepted(task_id).await {
            Ok(PullRequestLanding::NotInFlow) => self.merge_locally(task_id).await,
            Ok(PullRequestLanding::Merged) => {
                info!(%task_id, "Accepted task's pull request merged");
                self.close_task(task_id).await;
                self.finish(task_id).await;
            }
            Ok(PullRequestLanding::AlreadyMerged | PullRequestLanding::NothingToMerge) => self.finish(task_id).await,
            Ok(PullRequestLanding::Conflicting { base }) => self.resolve_conflict(task_id, "the pull request conflicts with its base branch", TerminalMode::MergePullRequest { base }).await,
            Ok(PullRequestLanding::Refused(reason)) => warn!(%reason, %task_id, "Accepted task's pull request was not merged; the worktree stays"),
            Err(failure) => warn!(%failure, %task_id, "Could not merge the pull request of an accepted task; it is tried again when the task moves"),
        }
    }

    async fn merge_locally(&mut self, task_id: Id) {
        match self.worktrees.merge_accepted(task_id).await {
            Ok(Merge::Merged) => {
                info!(%task_id, "Accepted task's branch merged into the project");
                self.close_task(task_id).await;
                self.finish(task_id).await;
            }
            Ok(Merge::AlreadyMerged | Merge::NothingToMerge) => self.finish(task_id).await,
            Ok(Merge::KeptWithChanges) => self.note_unsaved_changes(task_id),
            Ok(Merge::Conflicts(reason)) => self.resolve_conflict(task_id, &reason, TerminalMode::Merge).await,
            Ok(Merge::Refused(reason)) => warn!(%reason, %task_id, "Accepted task's branch was not merged; the worktree stays"),
            Err(failure) => warn!(%failure, %task_id, "Could not merge the branch of an accepted task"),
        }
    }

    fn note_unsaved_changes(&self, task_id: Id) {
        if !self.resolutions.contains_key(&task_id) {
            info!(%task_id, "Accepted task's branch is not merged: its worktree has unsaved changes");
        }
    }

    async fn resolve_conflict(&mut self, task_id: Id, reason: &str, mode: TerminalMode) {
        if self.resolutions.contains_key(&task_id) {
            return;
        }
        warn!(%reason, %task_id, "Accepted task's branch conflicts; a merge session resolves it");
        let resolution = match self.terminals.open(merge_opening(task_id, mode)).await {
            Ok(session) => Resolution::Running(session.id),
            Err(failure) => {
                warn!(%failure, %task_id, "Could not open a session to resolve the conflict");
                Resolution::Abandoned
            }
        };
        self.resolutions.insert(task_id, resolution);
    }

    async fn close_task(&self, task_id: Id) {
        if let Err(failure) = self.worktrees.close_merged(task_id).await {
            warn!(%failure, %task_id, "Could not close a task whose branch is merged");
        }
    }

    async fn finish(&mut self, task_id: Id) {
        if let Some(Resolution::Running(session_id)) = self.resolutions.remove(&task_id) {
            self.terminals.close(session_id, MERGE_SESSION_CLOSED).await;
        }
        self.release_worktree(task_id).await;
    }

    async fn follow_resolutions(&mut self) {
        let running: Vec<(Id, Id)> = self
            .resolutions
            .iter()
            .filter_map(|(task_id, resolution)| match resolution {
                Resolution::Running(session_id) => Some((*task_id, *session_id)),
                Resolution::Abandoned => None,
            })
            .collect();
        for (task_id, session_id) in running {
            self.clean_if_accepted(task_id).await;
            if matches!(self.resolutions.get(&task_id), Some(Resolution::Running(_))) && self.terminals.get(session_id).is_none() {
                warn!(%task_id, "The merge session ended and the branch still conflicts; the worktree stays");
                self.resolutions.insert(task_id, Resolution::Abandoned);
            }
        }
    }

    async fn release_worktree(&self, task_id: Id) {
        match self.worktrees.release(task_id).await {
            Ok(Release::KeptWithChanges) => info!(%task_id, "Accepted task keeps its worktree: it has unsaved changes"),
            Ok(Release::Removed) => info!(%task_id, "Accepted task's worktree removed"),
            Ok(Release::NoWorktree) => {}
            Err(failure) => warn!(%failure, %task_id, "Could not remove the worktree of an accepted task"),
        }
    }
}

fn merge_opening(task_id: Id, mode: TerminalMode) -> TerminalOpening {
    TerminalOpening {
        target: TerminalTarget::Task(task_id),
        step_id: None,
        skip_permissions: false,
        model: None,
        effort: None,
        mode,
        kind: SessionKind::Terminal,
    }
}

fn task_moved_by(event: &PublishedEvent) -> Option<Id> {
    if let Some(steps) = event.downcast::<StepStreamEvent>() {
        return Some(steps.task_id);
    }
    event.downcast::<TaskReviewEvent>().map(|review| review.task_id)
}
