use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use rekall_kernel::Result;
use tracing::warn;

use crate::worktree::{OrphanReport, WorktreeHost};

/// Clears the worktrees a crash left behind. A worktree is an orphan when its branch belongs to
/// no active task.
#[derive(Clone)]
pub struct OrphanReclaimService {
    host: Arc<dyn WorktreeHost>,
}

impl OrphanReclaimService {
    pub fn new(host: Arc<dyn WorktreeHost>) -> Self {
        Self { host }
    }

    /// Removes the orphans with nothing unsaved; the others are reported and left untouched.
    pub async fn reclaim_orphans(&self, project_folder: &Path, active_branches: &HashSet<String>) -> Result<OrphanReport> {
        let mut report = OrphanReport::default();
        for worktree in self.host.list(project_folder).await? {
            if active_branches.contains(&worktree.branch) {
                continue;
            }
            if self.host.status(&worktree).await?.has_unsaved_changes {
                warn!(branch = %worktree.branch, folder = %worktree.folder.display(), "Worktree left without a task still has unsaved changes");
                report.kept_with_changes.push(worktree);
                continue;
            }
            self.host.remove(&worktree).await?;
            report.removed.push(worktree);
        }
        Ok(report)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/orphan_reclaim_service_tests.rs"]
mod tests;
