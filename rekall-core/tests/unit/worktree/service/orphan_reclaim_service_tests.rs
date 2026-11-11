use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;

use super::*;
use crate::worktree::{BranchReview, MergeOutcome, Worktree, WorktreeStatus};

struct FakeHost {
    worktrees: Vec<(Worktree, bool)>,
    removed: Mutex<Vec<String>>,
}

#[async_trait]
impl WorktreeHost for FakeHost {
    async fn create(&self, _: &Path, _: &str, _: &Path, _: &str) -> Result<Worktree> {
        unreachable!("reclaiming never creates")
    }

    async fn remove(&self, worktree: &Worktree) -> Result<()> {
        self.removed.lock().unwrap().push(worktree.branch.clone());
        Ok(())
    }

    async fn list(&self, _: &Path) -> Result<Vec<Worktree>> {
        Ok(self.worktrees.iter().map(|(worktree, _)| worktree.clone()).collect())
    }

    async fn review_branch(&self, _: &Path, _: &str, _: Option<&Path>, _: &str) -> Result<Option<BranchReview>> {
        unreachable!("reclaiming never reviews a branch")
    }

    async fn merge_branch(&self, _: &Path, _: &str, _: &str) -> Result<MergeOutcome> {
        unreachable!("reclaiming never merges")
    }

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus> {
        let dirty = self.worktrees.iter().any(|(candidate, dirty)| candidate == worktree && *dirty);
        Ok(WorktreeStatus { has_unsaved_changes: dirty, commits_ahead: 0 })
    }
}

fn worktree(task: &str) -> Worktree {
    Worktree { project_folder: PathBuf::from("/work/vega"), folder: PathBuf::from(format!("/data/worktrees/vega/{task}")), branch: format!("rekall/{task}") }
}

fn service(worktrees: Vec<(Worktree, bool)>) -> (OrphanReclaimService, Arc<FakeHost>) {
    let host = Arc::new(FakeHost { worktrees, removed: Mutex::new(Vec::new()) });
    (OrphanReclaimService::new(host.clone()), host)
}

#[tokio::test]
async fn a_clean_worktree_without_an_active_task_is_removed_and_a_dirty_one_is_reported() {
    let (service, host) = service(vec![(worktree("clean"), false), (worktree("dirty"), true)]);

    let report = service.reclaim_orphans(Path::new("/work/vega"), &HashSet::new()).await.unwrap();

    assert_eq!(report.removed, vec![worktree("clean")]);
    assert_eq!(report.kept_with_changes, vec![worktree("dirty")]);
    assert_eq!(*host.removed.lock().unwrap(), vec!["rekall/clean".to_string()]);
}

#[tokio::test]
async fn a_worktree_whose_task_is_active_is_left_alone() {
    let (service, host) = service(vec![(worktree("running"), false)]);
    let active = HashSet::from(["rekall/running".to_string()]);

    let report = service.reclaim_orphans(Path::new("/work/vega"), &active).await.unwrap();

    assert_eq!(report, OrphanReport::default());
    assert!(host.removed.lock().unwrap().is_empty());
}
