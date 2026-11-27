use rekall_core::commit::StepSnapshotService;
use rekall_core::feedback::FeedbackService;
use rekall_core::review::TaskReviewService;
use rekall_core::step::TaskStepService;
use rekall_kernel::Id;
use tracing::warn;

/// Runs the review line of a task with no checklist while a session is on it, and gives back what
/// an ended session held. A failure is logged and never stops the session.
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

    /// When the line starts running, the files already modified in the project's folder are
    /// recorded. A session on a project alone has no task to mark.
    pub async fn mark_review_running(&self, task_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        match self.task_review.session_running(task_id, true).await {
            Ok(true) => self.step_snapshots.take(task_id, None).await,
            Ok(false) => {}
            Err(failed) => warn!(%failed, %task_id, "Could not mark session work running"),
        }
    }

    /// For a session that was given work: whatever it left running goes back to open.
    pub async fn release_running_steps(&self, task_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        if let Err(failed) = self.task_steps.release_all_running(task_id).await {
            warn!(%failed, %task_id, "Could not release the running steps of an ended session");
        }
    }

    /// The review line goes back to open, and the thread entries the session took and never
    /// answered wait for the next run.
    pub async fn release_review_and_thread(&self, task_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        if let Err(failed) = self.task_review.session_running(task_id, false).await {
            warn!(%failed, %task_id, "Could not release the review line of an ended session");
        }
        if let Err(failed) = self.feedback.release_running(task_id).await {
            warn!(%failed, %task_id, "Could not give back the thread entries of an ended session");
        }
    }
}
