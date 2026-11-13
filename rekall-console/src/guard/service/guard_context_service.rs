use std::path::Path;

use rekall_core::{in_read, Ctx, Tx};
use rekall_kernel::{jstr, RekallError, Result};
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
            let projects = repo::project::list_in_catalog_order(tx.db()).await?;
            let worktrees = repo::task::list_with_worktree(tx.db()).await?;
            let Some(project) = project_holding(&projects, &worktrees, &command.folder) else {
                return Ok::<_, RekallError>(self.view(None, false, None));
            };
            let running_step = running_step_of(&tx, project).await?;
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

fn project_holding<'a>(projects: &'a [project::Model], worktrees: &[task::Model], folder: &str) -> Option<&'a project::Model> {
    let own = projects.iter().filter_map(|project| Some((project, project.repo_folder.as_deref().filter(|repo_folder| !jstr::is_blank(repo_folder))?)));
    let in_worktree = worktrees.iter().filter_map(|task| {
        let project = projects.iter().find(|project| project.id == task.project_id)?;
        Some((project, task.worktree_folder.as_deref().filter(|worktree| !jstr::is_blank(worktree))?))
    });
    own.chain(in_worktree)
        .filter(|(_, repo_folder)| Path::new(folder).starts_with(jstr::strip(repo_folder)))
        .max_by_key(|(_, repo_folder)| Path::new(jstr::strip(repo_folder)).components().count())
        .map(|(project, _)| project)
}

async fn running_step_of(tx: &Tx, project: &project::Model) -> Result<Option<RunningStepView>> {
    for task in repo::task::list_of_project(tx.db(), project.id).await? {
        let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
        if let Some(position) = steps.iter().position(|step| step.state == TaskStepState::Running) {
            return Ok(Some(RunningStepView { number: position as u32 + 1, task_anchor: task.anchor() }));
        }
    }
    Ok(None)
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/service/guard_context_service_tests.rs"]
mod tests;
