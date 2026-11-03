//! `060`: the working tree as it stood when a step, or a task with no steps, started running:
//! one header per step (or per task) and one row per file that was already modified. A header
//! with no file rows is a clean tree, which is not the same as no snapshot.

use super::{changeset, exec};

changeset!(StepSnapshot, "060-step-snapshot", |manager| {
    exec(manager, &[
        "CREATE TABLE step_snapshot (
            id TEXT NOT NULL CONSTRAINT pk_step_snapshot PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            taken_at TEXT NOT NULL,
            CONSTRAINT fk_step_snapshot_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_snapshot_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_step_snapshot_step ON step_snapshot (step_id) WHERE step_id IS NOT NULL",
        "CREATE UNIQUE INDEX ux_step_snapshot_task ON step_snapshot (task_id) WHERE step_id IS NULL",
        "CREATE TABLE step_snapshot_file (
            snapshot_id TEXT NOT NULL,
            path TEXT NOT NULL,
            content_hash TEXT,
            CONSTRAINT pk_step_snapshot_file PRIMARY KEY (snapshot_id, path),
            CONSTRAINT fk_step_snapshot_file_snapshot FOREIGN KEY (snapshot_id) REFERENCES step_snapshot (id) ON DELETE CASCADE)",
    ])
    .await
});
