//! `039`: the routes between a task's steps and the visits a flow makes through them. A route
//! names its outcome once per step it leaves; deleting either step deletes the route, and
//! deleting a step deletes its visits.

use super::{changeset, exec};

changeset!(StepFlow, "039-step-flow", |manager| {
    exec(manager, &[
        "CREATE TABLE step_link (
            id TEXT NOT NULL CONSTRAINT pk_step_link PRIMARY KEY,
            task_id TEXT NOT NULL,
            from_step_id TEXT NOT NULL,
            to_step_id TEXT NOT NULL,
            outcome VARCHAR(60) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_step_link_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_from FOREIGN KEY (from_step_id) REFERENCES task_step (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_to FOREIGN KEY (to_step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_step_link_outcome ON step_link (from_step_id, outcome)",
        "CREATE INDEX ix_step_link_task ON step_link (task_id)",
        "CREATE TABLE step_visit (
            id TEXT NOT NULL CONSTRAINT pk_step_visit PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            entered_at TEXT NOT NULL,
            left_at TEXT,
            outcome VARCHAR(60),
            CONSTRAINT fk_step_visit_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_visit_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE UNIQUE INDEX ux_step_visit_ordinal ON step_visit (task_id, ordinal)",
    ])
    .await
});
