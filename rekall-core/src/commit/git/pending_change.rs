use rekall_kernel::{jstr, RekallError, Result};

use super::PendingChangeKind;

/// One entry of `git status --porcelain -z`: what the working tree holds that HEAD does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingChange {
    pub kind: PendingChangeKind,
    pub path: String,
}

impl PendingChange {
    pub fn new(kind: PendingChangeKind, path: &str) -> Self {
        Self { kind, path: path.to_string() }
    }

    /// Reads the output of `git status --porcelain -z`: entries end in NUL and paths are never
    /// quoted. A rename is the new path followed by the old one, and counts as the new path
    /// renamed plus the old path deleted, which is what staging it takes.
    pub fn parse_all(output: &str) -> Result<Vec<Self>> {
        let mut entries = output.split('\0').filter(|entry| !entry.is_empty());
        let mut changes = Vec::new();
        while let Some(entry) = entries.next() {
            let change = Self::parse(entry)?;
            let renamed = change.kind == PendingChangeKind::Renamed;
            changes.push(change);
            if !renamed {
                continue;
            }
            if let Some(old_path) = entries.next() {
                changes.push(Self::new(PendingChangeKind::Deleted, old_path));
            }
        }
        Ok(changes)
    }

    /// Reads one entry (`XY path`). An untracked file counts as added: it is about to be staged.
    pub fn parse(entry: &str) -> Result<Self> {
        if jstr::len(entry) < 4 {
            return Err(RekallError::illegal(format!("Unexpected git status line: '{entry}'")));
        }
        let mut chars = entry.chars();
        let index = chars.next().unwrap_or(' ');
        let worktree = chars.next().unwrap_or(' ');
        let path = jstr::suffix(entry, 3);
        if index == 'R' || worktree == 'R' {
            return Ok(Self::new(PendingChangeKind::Renamed, path));
        }
        if index == '?' || index == 'A' || worktree == 'A' {
            return Ok(Self::new(PendingChangeKind::Added, path));
        }
        if index == 'D' || worktree == 'D' {
            return Ok(Self::new(PendingChangeKind::Deleted, path));
        }
        Ok(Self::new(PendingChangeKind::Modified, path))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/git/pending_change_tests.rs"]
mod tests;
