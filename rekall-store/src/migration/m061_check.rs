//! `061`: the commands that have to pass before a claim is accepted, and what each run found. A
//! check belongs to a project, or to one of its steps; a result keeps the task so that a task with
//! no steps can have one too. Named `done_check` because `check` is a SQL keyword.

use super::{changeset, exec};

changeset!(DoneCheck, "061-done-check", |manager| {
    exec(manager, &[
        "CREATE TABLE done_check (
            id TEXT NOT NULL CONSTRAINT pk_done_check PRIMARY KEY,
            project_id TEXT NOT NULL,
            step_id TEXT,
            label VARCHAR(100) NOT NULL,
            command VARCHAR(2000) NOT NULL,
            folder VARCHAR(500),
            timeout_seconds INTEGER NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_done_check_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_done_check_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_done_check_project ON done_check (project_id, step_id, position)",
        "CREATE TABLE done_check_result (
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
            CONSTRAINT fk_done_check_result_step FOREIGN KEY (step_id) REFERENCES task_step (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_done_check_result_step ON done_check_result (task_id, step_id, ran_at)",
    ])
    .await
});
