//! `083`: a task's pull request remembers when it was merged, so an accepted task is merged once.

use super::{changeset, exec};

changeset!(PullRequestMerged, "083-pull-request-merged", |manager| {
    exec(manager, &["ALTER TABLE task_pull_request ADD COLUMN merged_at TEXT"]).await
});
