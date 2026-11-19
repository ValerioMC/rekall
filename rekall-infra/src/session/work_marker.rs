use rekall_core::commit::StepSnapshotService;
use rekall_core::feedback::FeedbackService;
use rekall_core::review::TaskReviewService;
use rekall_core::step::TaskStepService;
use rekall_kernel::{Id, Result};
use tracing::warn;

/// Moves a task's step to `RUNNING` and back, or its review line on a task with no checklist,
/// while a session is on it; a session on the task alone also gives back the thread entries it took
/// and never answered. A failure is logged and never stops the session. When the work goes
/// from open to running, the files already modified in the project's folder are recorded.
#[derive(Clone)]
pub struct WorkMarker {
    task_steps: TaskStepService,
    task_review: TaskReviewService,
    step_snapshots: StepSnapshotService,
    feedback: FeedbackService,
}

impl WorkMarker {
    pub fn new(task_steps: TaskStepService, task_review: TaskReviewService, step_snapshots: StepSnapshotService, feedback: FeedbackService) -> Self {
        Self { task_steps, task_review, step_snapshots, feedback }
    }

    /// A session on a project alone has no task to mark.
    pub async fn mark_running(&self, task_id: Option<Id>, step_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        let started: Result<bool> = match step_id {
            Some(step_id) => self.task_steps.mark_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, true).await,
        };
        match started {
            Ok(true) => self.step_snapshots.take(task_id, step_id).await,
            Ok(false) => {}
            Err(failed) => warn!(%failed, %task_id, "Could not mark session work running"),
        }
    }

    pub async fn release_running(&self, task_id: Option<Id>, step_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        let result = match step_id {
            Some(step_id) => self.task_steps.release_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, false).await.map(|_| ()),
        };
        if let Err(failed) = result {
            warn!(%failed, %task_id, "Could not release session work");
        }
        if step_id.is_none() {
            if let Err(failed) = self.feedback.release_running(task_id).await {
                warn!(%failed, %task_id, "Could not give back the thread entries of an ended session");
            }
        }
    }
}
