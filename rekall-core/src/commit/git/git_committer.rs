use std::path::{Path, PathBuf};

use rekall_kernel::{jstr, RekallError, Result};

use super::{PendingChange, display_path, run};
use super::git_command::COMMAND_TIMEOUT;
use crate::commit::model::work_tree_snapshot::{SnapshotEntry, WorkTreeSnapshot};

/// The one place that writes to a repository: stages everything and commits it, as whoever git
/// is configured to be in that folder.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitCommitter;

impl GitCommitter {
    /// Every change `git add -A` would pick up, untracked files listed one by one.
    pub async fn pending_changes(&self, folder: &Path) -> Result<Vec<PendingChange>> {
        let status = run(folder, &["status", "--porcelain", "-uall"], None, COMMAND_TIMEOUT).await?;
        if !status.ok() {
            return Err(RekallError::illegal(format!(
                "Could not read the working tree in {}: {}",
                display_path(folder),
                status.error()
            )));
        }
        status
            .stdout
            .split('\n')
            .filter(|line| !jstr::is_blank(line))
            .map(PendingChange::parse)
            .collect()
    }

    /// The pending changes with the hash of each file's content, read by git from the disk one
    /// file at a time so a large file is never held in memory.
    pub async fn work_tree_snapshot(&self, folder: &Path) -> Result<WorkTreeSnapshot> {
        let changes = self.pending_changes(folder).await?;
        let root = self.work_tree_root(folder).await?;
        let mut on_disk = Vec::with_capacity(changes.len());
        for change in &changes {
            on_disk.push(tokio::fs::metadata(root.join(&change.path)).await.is_ok_and(|found| found.is_file()));
        }
        let present: Vec<&str> = changes.iter().zip(&on_disk).filter(|(_, here)| **here).map(|(change, _)| change.path.as_str()).collect();
        let mut hashes = self.content_hashes(&root, &present).await?.into_iter();
        let entries = changes
            .iter()
            .zip(on_disk)
            .map(|(change, here)| SnapshotEntry {
                path: change.path.clone(),
                content_hash: if here { hashes.next() } else { None },
            })
            .collect();
        Ok(WorkTreeSnapshot { entries })
    }

    /// Porcelain paths are relative to the repository root, which is not always the folder.
    async fn work_tree_root(&self, folder: &Path) -> Result<PathBuf> {
        let top = run(folder, &["rev-parse", "--show-toplevel"], None, COMMAND_TIMEOUT).await?;
        if !top.ok() || jstr::is_blank(top.output()) {
            return Err(RekallError::illegal(format!(
                "Could not find the repository root of {}: {}",
                display_path(folder),
                top.error()
            )));
        }
        Ok(PathBuf::from(top.output()))
    }

    async fn content_hashes(&self, root: &Path, paths: &[&str]) -> Result<Vec<String>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let hashed = run(root, &["hash-object", "--stdin-paths"], Some(&paths.join("\n")), COMMAND_TIMEOUT).await?;
        let hashes: Vec<String> = hashed.stdout.lines().map(str::to_string).collect();
        if !hashed.ok() || hashes.len() != paths.len() {
            return Err(RekallError::illegal(format!(
                "Could not hash the modified files in {}: {}",
                display_path(root),
                hashed.error()
            )));
        }
        Ok(hashes)
    }

    /// Stages everything, commits it with `message`, and hands back the new tip's hash.
    pub async fn commit_all(&self, folder: &Path, message: &str) -> Result<String> {
        let shown = display_path(folder);
        let staged = run(folder, &["add", "-A"], None, COMMAND_TIMEOUT).await?;
        if !staged.ok() {
            return Err(RekallError::illegal(format!("Could not stage the changes in {shown}: {}", staged.error())));
        }
        let committed = run(folder, &["commit", "-q", "-F", "-"], Some(message), COMMAND_TIMEOUT).await?;
        if !committed.ok() {
            let reason = if jstr::is_blank(committed.error()) { committed.output() } else { committed.error() };
            return Err(RekallError::illegal(format!("git commit failed in {shown}: {reason}")));
        }
        let tip = run(folder, &["rev-parse", "HEAD"], None, COMMAND_TIMEOUT).await?;
        if !tip.ok() || jstr::is_blank(tip.output()) {
            return Err(RekallError::illegal(format!("Committed, but could not read the new tip in {shown}.")));
        }
        Ok(tip.output().to_string())
    }
}
