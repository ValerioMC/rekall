use std::path::Path;

use rekall_core::guard::GuardScope;
use rekall_core::{in_read, Ctx, Tx};
use rekall_kernel::{jstr, Result};
use rekall_store::{project, task};
use rekall_store::repository as repo;
use rekall_store::TaskStepState;

use crate::guard::{GuardContextCommand, GuardContextView, RunningStepView};

#[derive(Clone)]
pub struct GuardContextService {
    ctx: Ctx,
    protected_credential_references: Vec<String>,
}

impl GuardContextService {
    pub fn new(ctx: Ctx, protected_credential_references: Vec<String>) -> Self {
        Self { ctx, protected_credential_references }
    }

    pub async fn context_of(&self, command: GuardContextCommand) -> Result<GuardContextView> {
        in_read!(&self.ctx, |tx| {
            let launched_task = match command.scope {
                Some(GuardScope::Work(task_id)) => repo::task::find_by_id(tx.db(), task_id).await?,
                _ => None,
            };
            if let Some(task) = launched_task {
                let project = repo::project::require(tx.db(), task.project_id).await?;
                let running_step = running_step_of(&tx, &task).await?;
                return Ok(self.view(Some(project.anchor()), project.auto_commit, running_step));
            }
            let projects = repo::project::list_in_catalog_order(tx.db()).await?;
            let worktrees = repo::task::list_with_worktree(tx.db()).await?;
            let Some((project, task)) = project_holding(&projects, &worktrees, &command.folder) else {
                return Ok(self.view(None, false, None));
            };
            let running_step = match (command.scope, task) {
                (None, Some(task)) => running_step_of(&tx, task).await?,
                _ => None,
            };
            Ok(self.view(Some(project.anchor()), project.auto_commit, running_step))
        })
    }

    fn view(&self, project_anchor: Option<String>, auto_commit: bool, running_step: Option<RunningStepView>) -> GuardContextView {
        GuardContextView {
            project_anchor,
            auto_commit,
            running_step,
            protected_credential_references: self.protected_credential_references.clone(),
        }
    }
}

fn project_holding<'a>(
    projects: &'a [project::Model],
    worktrees: &'a [task::Model],
    folder: &str,
) -> Option<(&'a project::Model, Option<&'a task::Model>)> {
    let own = projects.iter().filter_map(|project| Some((project, None, project.repo_folder.as_deref()?)));
    let in_worktree = worktrees.iter().filter_map(|task| {
        let project = projects.iter().find(|project| project.id == task.project_id)?;
        Some((project, Some(task), task.worktree_folder.as_deref()?))
    });
    own.chain(in_worktree)
        .filter(|(_, _, root)| !jstr::is_blank(root) && Path::new(folder).starts_with(jstr::strip(root)))
        .max_by_key(|(_, _, root)| Path::new(jstr::strip(root)).components().count())
        .map(|(project, task, _)| (project, task))
}

async fn running_step_of(tx: &Tx, task: &task::Model) -> Result<Option<RunningStepView>> {
    let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
    Ok(steps.iter().position(|step| step.state == TaskStepState::Running)
        .map(|position| RunningStepView { number: position as u32 + 1, task_anchor: task.anchor() }))
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/service/guard_context_service_tests.rs"]
mod tests;
