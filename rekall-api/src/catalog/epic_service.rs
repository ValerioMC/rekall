use rekall_common::{Id, RekallError, Result};
use rekall_repository::repository as repo;
use rekall_service::{in_read, Ctx};

use crate::catalog::EpicActivityResponse;

/// What an epic's dashboard reads that the task list does not carry: what moved on its tasks.
#[derive(Clone)]
pub struct EpicService {
    ctx: Ctx,
}

impl EpicService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn activity(&self, epic_id: Id) -> Result<Vec<EpicActivityResponse>> {
        in_read!(&self.ctx, |tx| {
            let epic = repo::task::require(tx.db(), epic_id).await?;
            if !epic.is_epic() {
                return Err(RekallError::illegal(format!("'{}' is not an epic, so it has no activity of its own.", epic.label)));
            }
            let tasks = repo::task::list_children_of(tx.db(), epic.id).await?;
            let ids: Vec<Id> = tasks.iter().map(|task| task.id).collect();
            let steps = repo::task_step::list_of_tasks(tx.db(), ids.clone()).await?;
            let wrapups = repo::wrapup::list_of_tasks(tx.db(), ids.clone()).await?;
            let commits = repo::commit_reference::list_of_tasks(tx.db(), ids).await?;
            Ok(EpicActivityResponse::feed(&tasks, &steps, &wrapups, &commits))
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/catalog/epic_service_tests.rs"]
mod tests;
