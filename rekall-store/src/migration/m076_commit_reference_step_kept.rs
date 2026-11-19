//! `076`: a commit stays on its task when the step it was logged against is deleted, as token usage
//! does, and the table is indexed on both parents: it is read by task on every context load.

use super::{changeset, exec};

changeset!(CommitReferenceStepKept, "076-commit-reference-step-kept", |manager| {
    exec(manager, &[
        "CREATE TABLE commit_reference_kept (
            id TEXT NOT NULL CONSTRAINT pk_commit_reference PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            commit_hash VARCHAR(40) NOT NULL,
            comment VARCHAR(200) NOT NULL,
            created_at TEXT NOT NULL,
            diff TEXT,
            in_context BOOLEAN NOT NULL DEFAULT 0,
            CONSTRAINT fk_commit_reference_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_commit_reference_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE SET NULL)",
        "INSERT INTO commit_reference_kept (id, task_id, step_id, commit_hash, comment, created_at, diff, in_context)
            SELECT id, task_id, step_id, commit_hash, comment, created_at, diff, in_context FROM commit_reference",
        "DROP TABLE commit_reference",
        "ALTER TABLE commit_reference_kept RENAME TO commit_reference",
        "CREATE INDEX ix_commit_reference_task ON commit_reference (task_id)",
        "CREATE INDEX ix_commit_reference_step ON commit_reference (step_id)",
    ])
    .await
});
