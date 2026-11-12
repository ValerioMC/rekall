use rekall_kernel::{RekallError, Result};
use rekall_store::ConnectionKind;

use crate::worktree::model::pull_request_flow_site::PullRequestFlowSite;

/// The flow can be turned on: a folder to branch in, a branch to open pull requests against and
/// a task manager that has them.
pub fn require_ground(site: PullRequestFlowSite, task_manager: Option<ConnectionKind>) -> Result<()> {
    if !site.has_folder {
        return Err(RekallError::illegal("Set this project's folder before turning on the pull request flow."));
    }
    if !site.has_branch {
        return Err(RekallError::illegal("The pull request flow needs a main branch: name one, or point the folder at a git repository on a branch."));
    }
    match task_manager {
        Some(kind) if hosts_pull_requests(kind) => Ok(()),
        Some(_) => Err(RekallError::illegal("Only a GitHub or GitLab task manager has pull requests. Choose one before turning on the pull request flow.")),
        None => Err(RekallError::illegal("Choose a GitHub or GitLab connection as this project's task manager before turning on the pull request flow.")),
    }
}

/// A project whose flow is on keeps a task manager that has pull requests.
pub fn require_host_kept(task_manager_after: Option<ConnectionKind>) -> Result<()> {
    match task_manager_after {
        Some(kind) if hosts_pull_requests(kind) => Ok(()),
        _ => Err(RekallError::illegal("This project's pull request flow is on, and it needs a GitHub or GitLab task manager. Turn the flow off first.")),
    }
}

fn hosts_pull_requests(kind: ConnectionKind) -> bool {
    matches!(kind, ConnectionKind::Github | ConnectionKind::Gitlab)
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/rule/pull_request_flow_rules_tests.rs"]
mod tests;
