//! `085`: a task's link remembers the fingerprint of the content last written to its issue, so the
//! console can tell whether the task changed since. A link without one counts as not written yet.

use super::{changeset, exec};

changeset!(TaskTrackerSyncedContent, "085-task-tracker-synced-content", |manager| {
    exec(manager, &["ALTER TABLE task_tracker_link ADD COLUMN synced_fingerprint VARCHAR(16)"]).await
});
