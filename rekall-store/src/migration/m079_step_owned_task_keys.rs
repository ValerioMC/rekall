//! `079`: a row that names a step and its task agrees with the step about the task. The step's key
//! is also `(task_id, id)`, and the visits, routes, snapshots and check results that carry both
//! reference that pair. `token_usage` keeps its single-column key: `ON DELETE SET NULL` on a pair
//! would also clear its `task_id`. A file whose rows already disagree is refused when it opens, by
//! the foreign-key check that follows every migration.

use super::{changeset, exec};

changeset!(StepOwnedTaskKeys, "079-step-owned-task-keys", |manager| {
    exec(manager, &[
        "CREATE UNIQUE INDEX ux_task_step_task_id ON task_step (task_id, id)",
        "CREATE TABLE step_link_keyed (
            id TEXT NOT NULL CONSTRAINT pk_step_link PRIMARY KEY,
            task_id TEXT NOT NULL,
            from_step_id TEXT NOT NULL,
            to_step_id TEXT,
            outcome VARCHAR(60) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_step_link_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_from FOREIGN KEY (task_id, from_step_id) REFERENCES task_step (task_id, id) ON DELETE CASCADE,
            CONSTRAINT fk_step_link_to FOREIGN KEY (task_id, to_step_id) REFERENCES task_step (task_id, id) ON DELETE CASCADE)",
        "INSERT INTO step_link_keyed (id, task_id, from_step_id, to_step_id, outcome, position, created_at, updated_at) SELECT id, task_id, from_step_id, to_step_id, outcome, position, created_at, updated_at FROM step_link",
        "DROP TABLE step_link",
        "ALTER TABLE step_link_keyed RENAME TO step_link",
        "CREATE UNIQUE INDEX ux_step_link_outcome ON step_link (from_step_id, outcome)",
        "CREATE INDEX ix_step_link_task ON step_link (task_id)",
        "CREATE INDEX ix_step_link_to_step_id ON step_link (to_step_id)",
        "CREATE TABLE step_visit_keyed (
            id TEXT NOT NULL CONSTRAINT pk_step_visit PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            entered_at TEXT NOT NULL,
            left_at TEXT,
            outcome VARCHAR(60), report TEXT,
            CONSTRAINT fk_step_visit_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_visit_step FOREIGN KEY (task_id, step_id) REFERENCES task_step (task_id, id) ON DELETE CASCADE)",
        "INSERT INTO step_visit_keyed (id, task_id, step_id, ordinal, entered_at, left_at, outcome, report) SELECT id, task_id, step_id, ordinal, entered_at, left_at, outcome, report FROM step_visit",
        "DROP TABLE step_visit",
        "ALTER TABLE step_visit_keyed RENAME TO step_visit",
        "CREATE UNIQUE INDEX ux_step_visit_ordinal ON step_visit (task_id, ordinal)",
        "CREATE INDEX ix_step_visit_step_id ON step_visit (step_id)",
        "CREATE TABLE step_snapshot_keyed (
            id TEXT NOT NULL CONSTRAINT pk_step_snapshot PRIMARY KEY,
            task_id TEXT NOT NULL,
            step_id TEXT,
            taken_at TEXT NOT NULL,
            CONSTRAINT fk_step_snapshot_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_step_snapshot_step FOREIGN KEY (task_id, step_id) REFERENCES task_step (task_id, id) ON DELETE CASCADE)",
        "INSERT INTO step_snapshot_keyed (id, task_id, step_id, taken_at) SELECT id, task_id, step_id, taken_at FROM step_snapshot",
        "DROP TABLE step_snapshot",
        "ALTER TABLE step_snapshot_keyed RENAME TO step_snapshot",
        "CREATE UNIQUE INDEX ux_step_snapshot_step ON step_snapshot (step_id) WHERE step_id IS NOT NULL",
        "CREATE UNIQUE INDEX ux_step_snapshot_task ON step_snapshot (task_id) WHERE step_id IS NULL",
        "CREATE TABLE done_check_result_keyed (
            id TEXT NOT NULL CONSTRAINT pk_done_check_result PRIMARY KEY,
            check_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            step_id TEXT,
            passed INTEGER NOT NULL,
            exit_code INTEGER,
            output_tail TEXT NOT NULL,
            ran_at TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            CONSTRAINT fk_done_check_result_check FOREIGN KEY (check_id) REFERENCES done_check (id) ON DELETE CASCADE,
            CONSTRAINT fk_done_check_result_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_done_check_result_step FOREIGN KEY (task_id, step_id) REFERENCES task_step (task_id, id) ON DELETE CASCADE)",
        "INSERT INTO done_check_result_keyed (id, check_id, task_id, step_id, passed, exit_code, output_tail, ran_at, duration_ms) SELECT id, check_id, task_id, step_id, passed, exit_code, output_tail, ran_at, duration_ms FROM done_check_result",
        "DROP TABLE done_check_result",
        "ALTER TABLE done_check_result_keyed RENAME TO done_check_result",
        "CREATE INDEX ix_done_check_result_step ON done_check_result (task_id, step_id, ran_at)",
        "CREATE INDEX ix_done_check_result_step_id ON done_check_result (step_id)",
        "CREATE INDEX ix_done_check_result_check_id ON done_check_result (check_id)",
    ])
    .await
});
