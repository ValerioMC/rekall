use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;

use crate::worktree::model::pull_request_flow_site::PullRequestFlowSite;
use crate::worktree::rule::pull_request_flow_rules::require_ground;
use crate::Tx;

/// Decides whether a project may run the pull request flow, from the task manager it is bound to.
#[derive(Clone, Copy, Debug, Default)]
pub struct PullRequestFlowService;

impl PullRequestFlowService {
    pub async fn require_ground_in(&self, tx: &Tx, project_id: Id, site: PullRequestFlowSite) -> Result<()> {
        let task_manager = repo::project_connection::task_manager_of_project(tx.db(), project_id).await?.map(|(_, connection)| connection.kind);
        require_ground(site, task_manager)
    }
}
