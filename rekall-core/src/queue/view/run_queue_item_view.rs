use rekall_kernel::{Id, Instant};
use rekall_store::{project, run_queue_item, task, RunQueueItemState, SessionProvider};
use serde::Serialize;

use crate::claude::{CodexSessionOptions, StepProgress};
use crate::queue::{ClaudeQueueOptions, QueueLaunch, RunQueueWaitView};

/// One queued task, with enough of the task to name it without a second request, and its
/// released steps with the one being worked (none for a task with no checklist), and the
/// unaccepted tasks a waiting item depends on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueueItemView {
    pub id: Id,
    pub task_id: Id,
    pub task_title: String,
    pub task_code: String,
    pub project_label: String,
    pub anchor: String,
    pub position: i32,
    pub state: RunQueueItemState,
    #[serde(flatten)]
    pub launch: QueueLaunch,
    pub detail: Option<String>,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
    pub steps: Vec<StepProgress>,
    pub waiting_for: Vec<RunQueueWaitView>,
}

impl RunQueueItemView {
    pub fn of(item: &run_queue_item::Model, task: &task::Model, project: &project::Model, steps: Vec<StepProgress>, waiting_for: Vec<RunQueueWaitView>) -> Self {
        Self {
            id: item.id,
            task_id: task.id,
            task_title: task.title.clone(),
            task_code: task.code.clone(),
            project_label: project.label.clone(),
            anchor: task.anchor(),
            position: item.position,
            state: item.state,
            launch: QueueLaunch {
                provider: item.provider,
                claude_options: (item.provider == SessionProvider::Claude && item.claude_options_configured).then(|| ClaudeQueueOptions {
                    model: item.claude_model.clone(),
                    effort: item.claude_effort.clone(),
                }),
                codex_options: (item.provider == SessionProvider::Codex).then(|| CodexSessionOptions {
                    model: item.codex_model.clone(),
                    effort: item.codex_effort.clone(),
                    skip_permissions: item.codex_skip_permissions,
                }),
            },
            detail: item.detail.clone(),
            started_at: item.started_at,
            finished_at: item.finished_at,
            steps,
            waiting_for,
        }
    }
}
