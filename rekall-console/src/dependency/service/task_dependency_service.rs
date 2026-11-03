use rekall_core::dependency::{closes_a_cycle, TaskWaitService};
use rekall_core::queue::RunQueueService;
use rekall_core::{in_read, in_write, Ctx, Tx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task, task_dependency};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::dependency::{TaskDependencyCommand, TaskDependencyView};

#[derive(Clone)]
pub struct TaskDependencyService {
    ctx: Ctx,
    queue: RunQueueService,
}

impl TaskDependencyService {
    pub fn new(ctx: Ctx, queue: RunQueueService) -> Self {
        Self { ctx, queue }
    }

    pub async fn list_of(&self, task_id: Id) -> Result<Vec<TaskDependencyView>> {
        in_read!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            views_in(&tx, task_id).await
        })
    }

    pub async fn add(&self, task_id: Id, command: TaskDependencyCommand) -> Result<Vec<TaskDependencyView>> {
        let views = in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let dependency = repo::task::require(tx.db(), command.depends_on_task_id).await?;
            require_allowed(&tx, &task, &dependency).await?;
            task_dependency::Model::new(task.id, dependency.id, self.ctx.now()).into_active_model().insert(tx.db()).await?;
            views_in(&tx, task.id).await
        })?;
        self.queue.refresh_for_task(task_id).await?;
        Ok(views)
    }

    pub async fn remove(&self, task_id: Id, depends_on_task_id: Id) -> Result<Vec<TaskDependencyView>> {
        let views = in_write!(&self.ctx, |tx| {
            let dependency = repo::task_dependency::find_pair(tx.db(), task_id, depends_on_task_id)
                .await?
                .ok_or_else(|| RekallError::not_found_msg("That task does not depend on the other one."))?;
            task_dependency::Entity::delete_by_id(dependency.id).exec(tx.db()).await?;
            views_in(&tx, task_id).await
        })?;
        self.queue.refresh_for_task(task_id).await?;
        Ok(views)
    }
}

async fn views_in(tx: &Tx, task_id: Id) -> Result<Vec<TaskDependencyView>> {
    Ok(TaskWaitService::of_task_in(tx, task_id).await?.iter().map(TaskDependencyView::of).collect())
}

async fn require_allowed(tx: &Tx, task: &task::Model, dependency: &task::Model) -> Result<()> {
    task.require_workable("never takes part in a dependency")?;
    dependency.require_workable("never takes part in a dependency")?;
    require_same_project(task, dependency)?;
    require_not_already_added(tx, task, dependency).await?;
    require_no_cycle(tx, task, dependency).await
}

fn require_same_project(task: &task::Model, dependency: &task::Model) -> Result<()> {
    if task.project_id != dependency.project_id {
        return Err(RekallError::illegal(format!(
            "'{}' is in another project. A task depends only on a task of its own project.",
            dependency.label
        )));
    }
    Ok(())
}

async fn require_not_already_added(tx: &Tx, task: &task::Model, dependency: &task::Model) -> Result<()> {
    if repo::task_dependency::find_pair(tx.db(), task.id, dependency.id).await?.is_some() {
        return Err(RekallError::conflict(format!("'{}' already depends on '{}'.", task.label, dependency.label)));
    }
    Ok(())
}

async fn require_no_cycle(tx: &Tx, task: &task::Model, dependency: &task::Model) -> Result<()> {
    let existing: Vec<(Id, Id)> = repo::task_dependency::list_of_project(tx.db(), task.project_id)
        .await?
        .into_iter()
        .map(|row| (row.task_id, row.depends_on_task_id))
        .collect();
    if closes_a_cycle(&existing, task.id, dependency.id) {
        return Err(RekallError::illegal(format!(
            "'{}' cannot depend on '{}': that would make a task wait, directly or through others, for itself.",
            task.label, dependency.label
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/dependency/service/task_dependency_service_tests.rs"]
mod tests;
