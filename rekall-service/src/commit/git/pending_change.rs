use rekall_common::{jstr, RekallError, Result};

use super::PendingChangeKind;

/// One line of `git status --porcelain`: what the working tree holds that HEAD does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingChange {
    pub kind: PendingChangeKind,
    pub path: String,
}

impl PendingChange {
    pub fn new(kind: PendingChangeKind, path: &str) -> Self {
        Self { kind, path: path.to_string() }
    }

    /// Reads a porcelain v1 line (`XY path`, or `XY old -> new` for a rename). An untracked file
    /// counts as added: `git add -A` is about to stage it.
    pub fn parse(line: &str) -> Result<Self> {
        if jstr::len(line) < 4 {
            return Err(RekallError::illegal(format!("Unexpected git status line: '{line}'")));
        }
        let mut chars = line.chars();
        let index = chars.next().unwrap_or(' ');
        let worktree = chars.next().unwrap_or(' ');
        let path = jstr::suffix(line, 3);
        if index == 'R' || worktree == 'R' {
            let path = match path.find(" -> ") {
                Some(arrow) => &path[arrow + 4..],
                None => path,
            };
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
#[path = "../../../test/commit/git/pending_change_tests.rs"]
mod tests;
