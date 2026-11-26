use super::{changeset, exec};

changeset!(TaskBranchDelivery, "098-task-branch-delivery", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN last_work_provider VARCHAR(20) CHECK (last_work_provider IN ('CLAUDE', 'CODEX'))",
        "ALTER TABLE task_pull_request ADD COLUMN delivery_version BIGINT NOT NULL DEFAULT 0",
        "ALTER TABLE task_pull_request ADD COLUMN published_version BIGINT NOT NULL DEFAULT 0",
    ]).await
});
