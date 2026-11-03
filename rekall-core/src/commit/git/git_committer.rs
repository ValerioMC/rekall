use std::path::{Path, PathBuf};

use rekall_kernel::{jstr, RekallError, Result};

use super::{PendingChange, PendingChangeKind, display_path, run};
use super::git_command::COMMAND_TIMEOUT;
use crate::commit::model::work_tree_snapshot::{SnapshotEntry, WorkTreeSnapshot};

/// The one place that writes to a repository: stages named paths and commits only those, as
/// whoever git is configured to be in that folder.
#[derive(Clone, Copy, Debug, Default)]
pub struct GitCommitter;

impl GitCommitter {
    /// Every modified, new or deleted file, untracked files listed one by one.
    pub async fn pending_changes(&self, folder: &Path) -> Result<Vec<PendingChange>> {
        let status = run(folder, &["status", "--porcelain", "-z", "-uall"], None, COMMAND_TIMEOUT).await?;
        if !status.ok() {
            return Err(RekallError::illegal(format!(
                "Could not read the working tree in {}: {}",
                display_path(folder),
                status.error()
            )));
        }
        PendingChange::parse_all(&status.stdout)
    }

    /// The pending changes with the hash of each file's content.
    pub async fn work_tree_snapshot(&self, folder: &Path) -> Result<WorkTreeSnapshot> {
        let changes = self.pending_changes(folder).await?;
        let hashes = self.content_hashes_of(folder, &changes).await?;
        let entries = changes
            .into_iter()
            .zip(hashes)
            .map(|(change, content_hash)| SnapshotEntry { path: change.path, content_hash })
            .collect();
        Ok(WorkTreeSnapshot { entries })
    }

    /// One hash per change, in order, read by git from the disk file by file so a large file is
    /// never held in memory. A change with no file on disk has none.
    pub async fn content_hashes_of(&self, folder: &Path, changes: &[PendingChange]) -> Result<Vec<Option<String>>> {
        let root = self.work_tree_root(folder).await?;
        let mut on_disk = Vec::with_capacity(changes.len());
        for change in changes {
            on_disk.push(tokio::fs::metadata(root.join(&change.path)).await.is_ok_and(|found| found.is_file()));
        }
        let present: Vec<&str> =
            changes.iter().zip(&on_disk).filter(|(_, here)| **here).map(|(change, _)| change.path.as_str()).collect();
        let mut hashes = self.content_hashes(&root, &present).await?.into_iter();
        Ok(on_disk.into_iter().map(|here| if here { hashes.next() } else { None }).collect())
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

    /// Stages exactly `changes` and commits exactly them with `message`, leaving anything else
    /// that is modified or already staged where it is. Hands back the new tip's hash.
    pub async fn commit_paths(&self, folder: &Path, changes: &[PendingChange], message: &str) -> Result<String> {
        let shown = display_path(folder);
        let root = self.work_tree_root(folder).await?;
        let (deleted, present): (Vec<&PendingChange>, Vec<&PendingChange>) =
            changes.iter().partition(|change| change.kind == PendingChangeKind::Deleted);
        self.stage(&root, &["rm", "--cached", "--ignore-unmatch", "-q"], &deleted, &shown).await?;
        self.stage(&root, &["add"], &present, &shown).await?;
        let everything: Vec<&PendingChange> = changes.iter().collect();
        let committed = run(
            &root,
            &["--literal-pathspecs", "commit", "-q", "--only", "-m", message, "--pathspec-from-file=-", "--pathspec-file-nul"],
            Some(&nul_separated(&everything)),
            COMMAND_TIMEOUT,
        )
        .await?;
        if !committed.ok() {
            let reason = if jstr::is_blank(committed.error()) { committed.output() } else { committed.error() };
            return Err(RekallError::illegal(format!("git commit failed in {shown}: {reason}")));
        }
        let tip = run(&root, &["rev-parse", "HEAD"], None, COMMAND_TIMEOUT).await?;
        if !tip.ok() || jstr::is_blank(tip.output()) {
            return Err(RekallError::illegal(format!("Committed, but could not read the new tip in {shown}.")));
        }
        Ok(tip.output().to_string())
    }

    async fn stage(&self, root: &Path, command: &[&str], changes: &[&PendingChange], shown: &str) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }
        let mut arguments = vec!["--literal-pathspecs"];
        arguments.extend_from_slice(command);
        arguments.extend_from_slice(&["--pathspec-from-file=-", "--pathspec-file-nul"]);
        let staged = run(root, &arguments, Some(&nul_separated(changes)), COMMAND_TIMEOUT).await?;
        if !staged.ok() {
            return Err(RekallError::illegal(format!("Could not stage the changes in {shown}: {}", staged.error())));
        }
        Ok(())
    }
}

fn nul_separated(changes: &[&PendingChange]) -> String {
    changes.iter().map(|change| change.path.as_str()).collect::<Vec<_>>().join("\0")
}
