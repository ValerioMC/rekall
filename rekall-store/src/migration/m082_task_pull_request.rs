//! `082`: the pull request opened for a task's branch, one row per task that reached a claim on a
//! project with the pull request flow on, deleted with its task.

use super::{changeset, exec};

changeset!(TaskPullRequest, "082-task-pull-request", |manager| {
    exec(manager, &[
        "CREATE TABLE task_pull_request (
            task_id TEXT NOT NULL CONSTRAINT pk_task_pull_request PRIMARY KEY,
            number BIGINT,
            url VARCHAR(1000),
            failure VARCHAR(1000),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_task_pull_request_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
    ])
    .await
});
