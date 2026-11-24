//! `093`: the working set, the tasks a person keeps open, in the order they were opened, and the last
//! place seen in each project. A task leaves the set when it is deleted; a remembered task that is
//! deleted leaves its project at the task list.

use super::{changeset, exec};

changeset!(WorkingSet, "093-working-set", |manager| {
    exec(manager, &[
        "CREATE TABLE working_set_task (
            task_id TEXT NOT NULL CONSTRAINT pk_working_set_task PRIMARY KEY,
            position INTEGER NOT NULL,
            CONSTRAINT fk_working_set_task_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_working_set_task_position ON working_set_task (position)",
        "CREATE TABLE working_set_place (
            project_id TEXT NOT NULL CONSTRAINT pk_working_set_place PRIMARY KEY,
            task_id TEXT,
            CONSTRAINT fk_working_set_place_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_working_set_place_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE SET NULL)",
        "CREATE INDEX ix_working_set_place_task ON working_set_place (task_id)",
    ])
    .await
});
