//! `100`: a task remembers that its accepted branch is held back only by uncommitted files, so the
//! merge is tried again until the worktree is clean, whatever the task's status.

use super::{changeset, exec};

changeset!(MergeWaitsForCleanWorktree, "100-merge-waits-for-clean-worktree", |manager| {
    exec(manager, &["ALTER TABLE task ADD COLUMN merge_awaits_clean_worktree BOOLEAN NOT NULL DEFAULT 0"]).await
});
