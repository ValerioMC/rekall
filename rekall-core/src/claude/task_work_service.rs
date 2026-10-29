use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::TaskStepState;

use crate::{in_read, load, Ctx};

use super::{Remaining, TaskWork};

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
                return Ok(TaskWork {
                    remaining,
                    settled_steps: 0,
                    running_step_ids: Vec::new(),
                    last_settled_at: None,
                    next_opens_session: false,
                });
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
            let open_visit = repo::step_visit::list_of_task(tx.db(), task_id)
                .await?
                .into_iter()
                .find(|visit| visit.is_open())
                .map(|visit| visit.step_id);
            let next_to_work = checklist.iter().position(|step| match open_visit {
                Some(step_id) => step.id == step_id,
                None => step.state == TaskStepState::Open,
            });
            Ok::<_, RekallError>(TaskWork {
                remaining,
                settled_steps: checklist.iter().filter(|step| step.state.complete()).count(),
                running_step_ids: checklist.iter().filter(|step| step.state.running()).map(|step| step.id).collect(),
                last_settled_at: checklist.iter().filter_map(|step| step.completed_at()).max(),
                next_opens_session: next_to_work.is_some_and(|index| index > 0 && checklist[index].starts_session),
            })
        })
    }
}
