//! `040`: a route may lead to the end of the flow instead of to a step (`to_step_id` null), and a
//! step keeps the place the person dragged it to on the flow canvas. SQLite cannot drop a NOT NULL,
//! so `step_link` is rebuilt with its rows and indexes.

use super::{changeset, exec};

changeset!(FlowStartEnd, "040-flow-start-end", |manager| {
    exec(manager, &[
        "CREATE TABLE step_link_next (
            id TEXT NOT NULL CONSTRAINT pk_step_link PRIMARY KEY,
            task_id TEXT NOT NULL,
            from_step_id TEXT NOT NULL,
            to_step_id TEXT,
            outcome VARCHAR(60) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_step_link_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_from FOREIGN KEY (from_step_id) REFERENCES task_step (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_to FOREIGN KEY (to_step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "INSERT INTO step_link_next SELECT id, task_id, from_step_id, to_step_id, outcome, position, created_at, updated_at FROM step_link",
        "DROP TABLE step_link",
        "ALTER TABLE step_link_next RENAME TO step_link",
        "CREATE UNIQUE INDEX ux_step_link_outcome ON step_link (from_step_id, outcome)",
        "CREATE INDEX ix_step_link_task ON step_link (task_id)",
        "ALTER TABLE task_step ADD COLUMN flow_x INTEGER",
        "ALTER TABLE task_step ADD COLUMN flow_y INTEGER",
    ])
    .await
});
