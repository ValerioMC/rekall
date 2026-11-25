//! `094`: the last context fill of a task and of each of its steps, in a table of its own so a
//! write per turn does not move the `updated_at` of the task or the step. A row goes with the
//! task or the step it describes.

use super::{changeset, exec};

changeset!(ContextFill, "094-context-fill", |manager| {
    exec(manager, &[
        "CREATE TABLE context_fill (
            id TEXT NOT NULL CONSTRAINT pk_context_fill PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            used_tokens INTEGER NOT NULL,
            window_tokens INTEGER NOT NULL,
            recorded_at TEXT NOT NULL,
            CONSTRAINT fk_context_fill_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_context_fill_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_context_fill_task ON context_fill (task_id) WHERE step_id IS NULL",
        "CREATE UNIQUE INDEX ux_context_fill_task_step ON context_fill (task_id, step_id) WHERE step_id IS NOT NULL",
        "CREATE INDEX ix_context_fill_step ON context_fill (step_id)",
    ])
    .await
});
