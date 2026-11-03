//! `062`: every text a task's wrapup has had, newest numbered highest. The wrapup row stays one per
//! task; a version is written in the transaction that writes the text. Each existing wrapup starts
//! with its current text as version 1.

use super::{changeset, exec};

changeset!(WrapupVersion, "062-wrapup-version", |manager| {
    exec(manager, &[
        "CREATE TABLE wrapup_version (
            id TEXT NOT NULL CONSTRAINT pk_wrapup_version PRIMARY KEY,
            task_id TEXT NOT NULL,
            number INTEGER NOT NULL,
            body_markdown VARCHAR(20000) NOT NULL,
            written_by VARCHAR(20) NOT NULL,
            written_at TEXT NOT NULL,
            commit_hash VARCHAR(64),
            CONSTRAINT uq_wrapup_version_number UNIQUE (task_id, number),
            CONSTRAINT fk_wrapup_version_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "INSERT INTO wrapup_version (id, task_id, number, body_markdown, written_by, written_at, commit_hash)
            SELECT id, task_id, 1, body_markdown, written_by, updated_at, NULL FROM wrapup",
    ])
    .await
});
