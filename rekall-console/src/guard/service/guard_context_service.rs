use std::path::Path;

use rekall_core::claude::{Remaining, TaskWorkService};
use rekall_core::guard::{GuardLateChanges, GuardMerge, GuardScope};
use rekall_core::worktree::TaskWorktreeService;
use rekall_core::{in_read, Ctx, Tx};
use rekall_kernel::{jstr, Id, Result};
use rekall_store::{project, task};
use rekall_store::repository as repo;
use rekall_store::TaskStepState;

use crate::guard::{GuardContextCommand, GuardContextView, RunningStepView};

/// A task in its worktree whose work waits for review, before its files are read from git.
struct ClaimedWork {
    task_id: Id,
    task_anchor: String,
    step_to_claim: Option<u32>,
}

#[derive(Clone)]
pub struct GuardContextService {
    ctx: Ctx,
    protected_credential_references: Vec<String>,
    worktrees: TaskWorktreeService,
}

impl GuardContextService {
    pub fn new(ctx: Ctx, protected_credential_references: Vec<String>, worktrees: TaskWorktreeService) -> Self {
        Self { ctx, protected_credential_references, worktrees }
    }

    pub async fn context_of(&self, command: GuardContextCommand) -> Result<GuardContextView> {
        let merge_branch = match command.scope {
            Some(GuardScope::ResolveMerge(task_id)) => match self.worktrees.resolution_branch(task_id, Path::new(&command.folder)).await {
                Ok(branch) => branch,
                Err(failure) => {
                    tracing::warn!(%failure, %task_id, "Could not verify the merge worktree");
                    None
                }
            },
            _ => None,
        };
        let (mut view, claimed) = self.read_context(&command, merge_branch).await?;
        if let Some(claimed) = claimed {
            view.late_changes = self.late_changes_of(claimed).await;
        }
        Ok(view)
    }

    async fn read_context(&self, command: &GuardContextCommand, merge_branch: Option<String>) -> Result<(GuardContextView, Option<ClaimedWork>)> {
        in_read!(&self.ctx, |tx| {
            let launched_task = match command.scope {
                Some(GuardScope::Work(task_id) | GuardScope::ResolveMerge(task_id)) => repo::task::find_by_id(tx.db(), task_id).await?,
                _ => None,
            };
            if let Some(task) = launched_task {
                let project = repo::project::require(tx.db(), task.project_id).await?;
                let mut view = self.view(Some(project.anchor()), project.auto_commit, None);
                let mut claimed = None;
                if matches!(command.scope, Some(GuardScope::Work(_))) {
                    view.running_step = running_step_of(&tx, &task).await?;
                    claimed = claimed_work_of(&tx, &project, &task, &command.folder).await?;
                }
                if let Some(branch) = merge_branch {
                    if TaskWorkService::read_in(&tx, task.id).await?.remaining == Remaining::Accepted {
                        view.merge = Some(GuardMerge { branch, pull_request: project.pull_request_flow });
                    }
                }
                return Ok((view, claimed));
            }
            let projects = repo::project::list_in_catalog_order(tx.db()).await?;
            let worktrees = repo::task::list_with_worktree(tx.db()).await?;
            let Some((project, task)) = project_holding(&projects, &worktrees, &command.folder) else {
                return Ok((self.view(None, false, None), None));
            };
            let (running_step, claimed) = match (command.scope, task) {
                (None, Some(task)) => (running_step_of(&tx, task).await?, claimed_work_of(&tx, project, task, &command.folder).await?),
                _ => (None, None),
            };
            Ok((self.view(Some(project.anchor()), project.auto_commit, running_step), claimed))
        })
    }

    /// A worktree git cannot read is reported as clean: the guard's other rules must still reach the session.
    async fn late_changes_of(&self, claimed: ClaimedWork) -> Option<GuardLateChanges> {
        let ClaimedWork { task_id, task_anchor, step_to_claim } = claimed;
        match self.worktrees.unsaved_files(task_id).await {
            Ok(files) => (!files.is_empty()).then_some(GuardLateChanges { task_anchor, files, step_to_claim }),
            Err(failure) => {
                tracing::warn!(%failure, %task_id, "Could not read the files left in a worktree");
                None
            }
        }
    }

    fn view(&self, project_anchor: Option<String>, auto_commit: bool, running_step: Option<RunningStepView>) -> GuardContextView {
        GuardContextView {
            project_anchor,
            auto_commit,
            merge: None,
            running_step,
            late_changes: None,
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

async fn claimed_work_of(tx: &Tx, project: &project::Model, task: &task::Model, folder: &str) -> Result<Option<ClaimedWork>> {
    let inside_worktree = task.own_worktree().is_some_and(|worktree| Path::new(folder).starts_with(jstr::strip(worktree)));
    if !project.auto_commit || !inside_worktree || TaskWorkService::read_in(tx, task.id).await?.remaining != Remaining::Claimed {
        return Ok(None);
    }
    Ok(Some(ClaimedWork { task_id: task.id, task_anchor: task.anchor(), step_to_claim: last_claimed_step_of(tx, task).await? }))
}

async fn last_claimed_step_of(tx: &Tx, task: &task::Model) -> Result<Option<u32>> {
    let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
    Ok(steps.iter().enumerate()
        .filter(|(_, step)| step.state == TaskStepState::Claimed)
        .max_by_key(|(_, step)| step.claimed_at)
        .map(|(position, _)| position as u32 + 1))
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/service/guard_context_service_tests.rs"]
mod tests;
