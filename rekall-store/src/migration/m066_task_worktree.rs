//! `066`: the folder of the git worktree a task works in when the run queue runs tasks in
//! parallel. Absent means the task works in its project's own folder.

use super::{changeset, exec};

changeset!(TaskWorktree, "066-task-worktree", |manager| {
    exec(manager, &["ALTER TABLE task ADD COLUMN worktree_folder VARCHAR(500)"]).await
});
