//! `001-core.yaml`: the first schema, with environments and single-owner documents, both of which
//! 002 takes apart again.

use super::{changeset, exec};

changeset!(Environment, "001-environment", |manager| {
    exec(manager, &["CREATE TABLE environment (
        id TEXT NOT NULL CONSTRAINT pk_environment PRIMARY KEY,
        label VARCHAR(160) NOT NULL,
        namespace VARCHAR(120),
        kubeconfig_path VARCHAR(500),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_environment_label UNIQUE (label))"])
    .await
});

changeset!(Project, "001-project", |manager| {
    exec(manager, &["CREATE TABLE project (
        id TEXT NOT NULL CONSTRAINT pk_project PRIMARY KEY,
        name VARCHAR(120) NOT NULL,
        status VARCHAR(20) NOT NULL,
        description VARCHAR(100000),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT uq_project_name UNIQUE (name))"])
    .await
});

changeset!(Task, "001-task", |manager| {
    exec(manager, &["CREATE TABLE task (
        id TEXT NOT NULL CONSTRAINT pk_task PRIMARY KEY,
        name VARCHAR(160) NOT NULL,
        status VARCHAR(20) NOT NULL,
        description VARCHAR(100000),
        project_id TEXT NOT NULL,
        environment_id TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        CONSTRAINT fk_task_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
        CONSTRAINT fk_task_environment FOREIGN KEY (environment_id) REFERENCES environment (id) ON DELETE RESTRICT,
        CONSTRAINT uq_task_project_name UNIQUE (project_id, name))"])
    .await
});

changeset!(Document, "001-document", |manager| {
    exec(manager, &[
        "CREATE TABLE document (
            id TEXT NOT NULL CONSTRAINT pk_document PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            kind VARCHAR(40) NOT NULL,
            body_markdown VARCHAR(100000) NOT NULL,
            source_path VARCHAR(500),
            position INTEGER NOT NULL,
            project_id TEXT,
            task_id TEXT,
            environment_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_document_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_document_task FOREIGN KEY (task_id) REFERENCES task (id) ON DELETE CASCADE,
            CONSTRAINT fk_document_environment FOREIGN KEY (environment_id) REFERENCES environment (id) ON DELETE CASCADE,
            CONSTRAINT ck_document_single_owner CHECK (
                (CASE WHEN project_id IS NULL THEN 0 ELSE 1 END)
              + (CASE WHEN task_id IS NULL THEN 0 ELSE 1 END)
              + (CASE WHEN environment_id IS NULL THEN 0 ELSE 1 END) = 1))",
        "CREATE INDEX idx_document_task ON document (task_id)",
        "CREATE INDEX idx_document_project ON document (project_id)",
    ])
    .await
});
