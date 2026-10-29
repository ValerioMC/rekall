//! `055`: a task can be an epic, and any other task of its project can belong to one.

use super::{changeset, exec};

changeset!(Epic, "055-epic", |manager| {
    exec(manager, &[
        "ALTER TABLE task ADD COLUMN kind VARCHAR(20) NOT NULL DEFAULT 'TASK'",
        "ALTER TABLE task ADD COLUMN epic_id TEXT CONSTRAINT fk_task_epic REFERENCES task (id) ON DELETE SET NULL",
        "CREATE INDEX idx_task_epic ON task (epic_id)",
    ])
    .await
});
