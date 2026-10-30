use rekall_core::review::TaskReviewService;
use rekall_core::step::TaskStepService;
use rekall_kernel::Id;
use tracing::warn;

/// Moves a task's step to `RUNNING` and back, or its review line on a task with no checklist,
/// while a session is on it. A failure is logged and never stops the session.
#[derive(Clone)]
pub struct WorkMarker {
    task_steps: TaskStepService,
    task_review: TaskReviewService,
}

impl WorkMarker {
    pub fn new(task_steps: TaskStepService, task_review: TaskReviewService) -> Self {
        Self { task_steps, task_review }
    }

    /// A session on a project alone has no task to mark.
    pub async fn mark_running(&self, task_id: Option<Id>, step_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        let result = match step_id {
            Some(step_id) => self.task_steps.mark_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, true).await,
        };
        if let Err(failed) = result {
            warn!(%failed, %task_id, "Could not mark session work running");
        }
    }

    pub async fn release_running(&self, task_id: Option<Id>, step_id: Option<Id>) {
        let Some(task_id) = task_id else { return };
        let result = match step_id {
            Some(step_id) => self.task_steps.release_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, false).await,
        };
        if let Err(failed) = result {
            warn!(%failed, %task_id, "Could not release session work");
        }
    }
}
