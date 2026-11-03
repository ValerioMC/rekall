//! `064`: a task that waits for another one to be done. Both rows go with their task.

use super::{changeset, exec};

changeset!(TaskDependency, "064-task-dependency", |manager| {
    exec(manager, &[
        "CREATE TABLE task_dependency (
            id TEXT NOT NULL CONSTRAINT pk_task_dependency PRIMARY KEY,
            task_id TEXT NOT NULL,
            depends_on_task_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            CONSTRAINT fk_task_dependency_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_task_dependency_on FOREIGN KEY (depends_on_task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT uq_task_dependency UNIQUE (task_id, depends_on_task_id))",
        "CREATE INDEX ix_task_dependency_on ON task_dependency (depends_on_task_id)",
    ])
    .await
});
