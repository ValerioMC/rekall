//! `080`: the issue a task was written to leaves the task row for a table of its own, one row per
//! task that was ever pushed, deleted with its task.

use super::{changeset, exec};

changeset!(TaskTrackerLink, "080-task-tracker-link", |manager| {
    exec(manager, &[
        "CREATE TABLE task_tracker_link (
            task_id TEXT NOT NULL CONSTRAINT pk_task_tracker_link PRIMARY KEY,
            issue_key VARCHAR(200),
            issue_url VARCHAR(1000),
            sync_error VARCHAR(1000),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_task_tracker_link_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "INSERT INTO task_tracker_link (task_id, issue_key, issue_url, sync_error, created_at, updated_at)
            SELECT id, tracker_issue_key, tracker_issue_url, tracker_sync_error, updated_at, updated_at
            FROM task
            WHERE tracker_issue_key IS NOT NULL OR tracker_issue_url IS NOT NULL OR tracker_sync_error IS NOT NULL",
        "ALTER TABLE task DROP COLUMN tracker_issue_key",
        "ALTER TABLE task DROP COLUMN tracker_issue_url",
        "ALTER TABLE task DROP COLUMN tracker_sync_error",
    ])
    .await
});
