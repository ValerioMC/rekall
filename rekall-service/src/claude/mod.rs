//! What the terminal and the run queue in `rekall-claude` need from the entities, read here so
//! that module never touches a repository.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::TaskStepState;

use crate::{in_read, load, Ctx};

/// Whether a session has anything to do on the task, and if not, why not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remaining {
    /// An open or running step, or a stepless task not yet claimed.
    Open,
    /// Everything is claimed and at least part of it waits for review.
    Claimed,
    /// Everything has been accepted.
    Accepted,
}

/// `settled_steps` counts the claimed and done steps, so a rise between two reads is a claim;
/// `running_step_ids` are the steps a session has marked running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWork {
    pub remaining: Remaining,
    pub settled_steps: usize,
    pub running_step_ids: Vec<Id>,
}

impl TaskWork {
    pub fn has_work(&self) -> bool {
        self.remaining == Remaining::Open
    }
}

/// Reads how much of a task is left for a session: from its non-draft steps when it has a
/// checklist, from its review line when it has none.
#[derive(Clone)]
pub struct TaskWorkService {
    ctx: Ctx,
}

impl TaskWorkService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn read(&self, task_id: Id) -> Result<TaskWork> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let checklist: Vec<_> = load::steps_of(tx.db(), task_id)
                .await?
                .into_iter()
                .filter(|step| !step.state.draft())
                .collect();
            if checklist.is_empty() {
                let remaining = match task.review_state {
                    TaskStepState::Claimed => Remaining::Claimed,
                    TaskStepState::Done => Remaining::Accepted,
                    _ => Remaining::Open,
                };
                return Ok(TaskWork { remaining, settled_steps: 0, running_step_ids: Vec::new() });
            }
            let work_left = checklist
                .iter()
                .any(|step| step.state == TaskStepState::Open || step.state.running());
            let all_done = checklist.iter().all(|step| step.state == TaskStepState::Done);
            let remaining = if work_left {
                Remaining::Open
            } else if all_done {
                Remaining::Accepted
            } else {
                Remaining::Claimed
            };
            Ok::<_, RekallError>(TaskWork {
                remaining,
                settled_steps: checklist.iter().filter(|step| step.state.complete()).count(),
                running_step_ids: checklist.iter().filter(|step| step.state.running()).map(|step| step.id).collect(),
            })
        })
    }
}

/// What the PTY needs to start: the folder to start `claude` in and the `/rk` anchors to load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalLaunch {
    pub task_id: Id,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: String,
    pub task_title: String,
}

#[derive(Clone)]
pub struct TerminalLaunchService {
    ctx: Ctx,
}

impl TerminalLaunchService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The folder is the project's `repo_folder`; a project without one is refused here.
    pub async fn resolve(&self, task_id: Id) -> Result<TerminalLaunch> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let Some(folder) = project.repo_folder.as_deref().filter(|f| !jstr::is_blank(f)) else {
                return Err(RekallError::illegal(
                    "Set this project's folder on its page before opening a terminal here.",
                ));
            };
            Ok(TerminalLaunch {
                task_id,
                anchors: format!("project:{} task:{}", project.label, task.label),
                working_dir: jstr::strip(folder).to_string(),
                project_label: project.label.clone(),
                task_label: task.label.clone(),
                task_title: task.title.clone(),
            })
        })
    }
}
