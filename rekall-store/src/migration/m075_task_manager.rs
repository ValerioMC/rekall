//! `075`: a project names one of its connection bindings as its task manager and the branch its
//! accepted work is merged into, and a task remembers the issue it was pushed to.

use super::{changeset, exec};

changeset!(TaskManager, "075-task-manager", |manager| {
    exec(manager, &[
        "ALTER TABLE project_connection ADD COLUMN task_manager BOOLEAN NOT NULL DEFAULT 0",
        "ALTER TABLE project ADD COLUMN main_branch VARCHAR(255)",
        "ALTER TABLE task ADD COLUMN tracker_issue_key VARCHAR(200)",
        "ALTER TABLE task ADD COLUMN tracker_issue_url VARCHAR(1000)",
        "ALTER TABLE task ADD COLUMN tracker_sync_error VARCHAR(1000)",
    ])
    .await
});
