//! `002-documents-on-many-tasks.yaml`: a note stops belonging to one owner and is linked to any
//! number of tasks; environments go.

use super::{changeset, exec, rebuild};

changeset!(DocumentTask, "002-document-task", |manager| {
    exec(manager, &[
        "CREATE TABLE document_task (
            document_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            CONSTRAINT pk_document_task PRIMARY KEY (document_id, task_id),
            CONSTRAINT fk_document_task_document FOREIGN KEY (document_id) REFERENCES document (id) ON DELETE CASCADE,
            CONSTRAINT fk_document_task_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE)",
        "CREATE INDEX idx_document_task_task ON document_task (task_id)",
    ])
    .await
});

changeset!(CarryTaskNotes, "002-carry-task-notes", |manager| {
    exec(manager, &[
        "INSERT INTO document_task (document_id, task_id, position)
         SELECT id, task_id, position FROM document WHERE task_id IS NOT NULL",
    ])
    .await
});

changeset!(DropOwnerlessNotes, "002-drop-ownerless-notes", |manager| {
    exec(manager, &["DELETE FROM document WHERE task_id IS NULL"]).await
});

// Drops the three owner keys, the single-owner check, both indexes and the four columns.
changeset!(DocumentLosesItsOwners, "002-document-loses-its-owners", |manager| {
    rebuild(
        manager,
        "document",
        "id TEXT NOT NULL CONSTRAINT pk_document PRIMARY KEY,
         title VARCHAR(255) NOT NULL,
         kind VARCHAR(40) NOT NULL,
         body_markdown VARCHAR(100000) NOT NULL,
         source_path VARCHAR(500),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL",
        "id, title, kind, body_markdown, source_path, created_at, updated_at",
        &[],
    )
    .await
});

changeset!(DropEnvironment, "002-drop-environment", |manager| {
    rebuild(
        manager,
        "task",
        "id TEXT NOT NULL CONSTRAINT pk_task PRIMARY KEY,
         name VARCHAR(160) NOT NULL,
         status VARCHAR(20) NOT NULL,
         description VARCHAR(100000),
         project_id TEXT NOT NULL,
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         CONSTRAINT fk_task_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
         CONSTRAINT uq_task_project_name UNIQUE (project_id, name)",
        "id, name, status, description, project_id, created_at, updated_at",
        &[],
    )
    .await?;
    exec(manager, &["DROP TABLE environment"]).await
});
