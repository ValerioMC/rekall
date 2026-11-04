//! `067`: a deviation from the plan becomes a record of its task instead of a note titled
//! `step-N-changes.md`. One per task and step; it goes with its task. Each such note that sits on one
//! task is moved here and removed from the notes; one shared by several tasks stays a note.

use super::{changeset, exec};

changeset!(TaskDeviation, "067-task-deviation", |manager| {
    exec(manager, &[
        "CREATE TABLE task_deviation (
            id TEXT NOT NULL CONSTRAINT pk_task_deviation PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_number INTEGER NOT NULL,
            body_markdown VARCHAR(100000) NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_task_deviation_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT uq_task_deviation_step UNIQUE (task_id, step_number))",
        "CREATE TEMP TABLE deviation_note AS
            SELECT d.id AS id, MIN(l.task_id) AS task_id,
                   CAST(substr(lower(trim(d.title)), 6, length(trim(d.title)) - 16) AS INTEGER) AS step_number
            FROM document d JOIN document_task l ON l.document_id = d.id
            WHERE lower(trim(d.title)) GLOB 'step-[0-9]*-changes.md'
              AND substr(lower(trim(d.title)), 6, length(trim(d.title)) - 16) NOT GLOB '*[^0-9]*'
              AND length(trim(d.title)) > 16
            GROUP BY d.id HAVING COUNT(*) = 1",
        "INSERT OR IGNORE INTO task_deviation (id, task_id, step_number, body_markdown, created_at, updated_at)
            SELECT d.id, n.task_id, n.step_number, d.body_markdown, d.created_at, d.updated_at
            FROM deviation_note n JOIN document d ON d.id = n.id WHERE n.step_number > 0",
        "DELETE FROM document WHERE id IN (SELECT id FROM task_deviation)",
        "DROP TABLE deviation_note",
    ])
    .await
});
