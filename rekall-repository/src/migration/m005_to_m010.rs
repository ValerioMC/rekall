//! `005` to `010`: the wrapup, time tracking, the project's blueprint and folder, and the step
//! checklist with its state line.

use super::{changeset, exec};

changeset!(Wrapup, "005-wrapup", |manager| {
    exec(manager, &["CREATE TABLE wrapup (
        id TEXT NOT NULL CONSTRAINT pk_wrapup PRIMARY KEY,
        task_id TEXT NOT NULL,
        body_markdown VARCHAR(20000) NOT NULL,
        written_by VARCHAR(20) NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_wrapup_task UNIQUE (task_id),
        CONSTRAINT fk_wrapup_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)"])
    .await
});

changeset!(TimeEntry, "006-time-entry", |manager| {
    exec(manager, &["CREATE TABLE time_entry (
        id TEXT NOT NULL CONSTRAINT pk_time_entry PRIMARY KEY,
        task_id TEXT NOT NULL,
        started_at TEXT NOT NULL,
        stopped_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT fk_time_entry_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)"])
    .await
});

changeset!(ProjectBlueprint, "007-project-blueprint", |manager| {
    exec(manager, &["ALTER TABLE project ADD COLUMN blueprint_markdown VARCHAR(100000)"]).await
});

changeset!(ProjectRepoFolder, "008-project-repo-folder", |manager| {
    exec(manager, &["ALTER TABLE project ADD COLUMN repo_folder VARCHAR(1000)"]).await
});

changeset!(TaskStep, "009-task-step", |manager| {
    exec(manager, &[
        "CREATE TABLE task_step (
            id TEXT NOT NULL CONSTRAINT pk_task_step PRIMARY KEY,
            task_id TEXT NOT NULL,
            title VARCHAR(200) NOT NULL,
            body_markdown VARCHAR(20000),
            done BOOLEAN NOT NULL DEFAULT 0,
            done_at TEXT,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_task_step_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX idx_task_step_task ON task_step (task_id, position)",
    ])
    .await
});

changeset!(TaskStepState, "010-task-step-state", |manager| {
    exec(manager, &[
        "ALTER TABLE task_step ADD COLUMN state VARCHAR(20) NOT NULL DEFAULT 'OPEN'",
        "ALTER TABLE task_step ADD COLUMN running_at TEXT",
        "ALTER TABLE task_step ADD COLUMN claimed_at TEXT",
        "UPDATE task_step SET state = 'DONE' WHERE done = 1",
        "ALTER TABLE task_step DROP COLUMN done",
    ])
    .await
});
