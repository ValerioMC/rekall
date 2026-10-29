use rekall_common::{Result, RekallError};
use rekall_store::task;
use rekall_store::repository as repo;

use super::{EpicJoined, TaskStreamEvent};
use crate::{in_write, load, Ctx, Tx};

/// Files a task under an epic of its project for a session: the one catalog write on a task a
/// session makes after creating it. It moves a task between epics, never out of one.
#[derive(Clone)]
pub struct EpicMembershipService {
    ctx: Ctx,
}

impl EpicMembershipService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn join(&self, project_label: Option<&str>, task_label: &str, epic_label: &str) -> Result<EpicJoined> {
        in_write!(&self.ctx, |tx| {
            let before = load::resolve_task(tx.db(), project_label, task_label).await?;
            let project = load::project_of(tx.db(), &before).await?;
            let epic = load::resolve_task(tx.db(), Some(&project.label), epic_label).await?;
            before.require_joinable(&epic)?;
            let left = self.epic_left(&tx, &before, &epic).await?;
            let task = task::Model { epic_id: Some(epic.id), ..before.clone() };
            repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
            tx.publish(TaskStreamEvent { task_id: before.id, project_id: project.id });
            Ok::<_, RekallError>(EpicJoined {
                task_anchor: before.anchor(&project.label),
                epic_anchor: epic.anchor(&project.label),
                epic_title: epic.title,
                left,
            })
        })
    }

    /// The label of the epic the task is moved out of; `None` when it was in none or already in this one.
    async fn epic_left(&self, tx: &Tx, task: &task::Model, epic: &task::Model) -> Result<Option<String>> {
        match task.epic_id.filter(|current| *current != epic.id) {
            Some(current) => Ok(Some(repo::task::require(tx.db(), current).await?.label)),
            None => Ok(None),
        }
    }
}
