use std::path::{Path, PathBuf};

use crate::worktree::{Worktree, BRANCH_PREFIX};

/// The worktrees of `git worktree list --porcelain` that sit on a branch Rekall names. The
/// repository's own folder, a detached worktree and anyone else's branch are left out.
pub fn parse_listing(project_folder: &Path, porcelain: &str) -> Vec<Worktree> {
    porcelain
        .split("\n\n")
        .filter_map(|block| worktree_of(project_folder, block))
        .collect()
}

fn worktree_of(project_folder: &Path, block: &str) -> Option<Worktree> {
    let folder = block.lines().find_map(|line| line.strip_prefix("worktree "))?;
    let reference = block.lines().find_map(|line| line.strip_prefix("branch "))?;
    let branch = reference.strip_prefix("refs/heads/")?;
    branch.starts_with(BRANCH_PREFIX).then(|| Worktree {
        project_folder: project_folder.to_path_buf(),
        folder: PathBuf::from(folder),
        branch: branch.to_string(),
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/rule/worktree_listing_tests.rs"]
mod tests;
