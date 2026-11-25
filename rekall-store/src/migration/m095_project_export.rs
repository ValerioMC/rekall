//! `095`: the exports of a project's pages as a PDF or Word file, done or failed. The file's bytes
//! sit in a table of their own so listing exports never reads one. Both rows go with the project.

use super::{changeset, exec};

changeset!(ProjectExport, "095-project-export", |manager| {
    exec(manager, &[
        "CREATE TABLE project_export (
            id TEXT NOT NULL CONSTRAINT pk_project_export PRIMARY KEY,
            project_id TEXT NOT NULL,
            format VARCHAR(20) NOT NULL,
            status VARCHAR(20) NOT NULL,
            summary VARCHAR(300) NOT NULL,
            file_name VARCHAR(255) NOT NULL,
            size_bytes INTEGER NOT NULL,
            diagrams_missing INTEGER NOT NULL,
            failure_kind VARCHAR(20),
            failure_message VARCHAR(1000),
            created_at TEXT NOT NULL,
            CONSTRAINT fk_project_export_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT ck_project_export_format CHECK (format IN ('PDF', 'DOCX')),
            CONSTRAINT ck_project_export_status CHECK (status IN ('DONE', 'FAILED')),
            CONSTRAINT ck_project_export_failure_kind CHECK (failure_kind IS NULL OR failure_kind IN ('RENDER', 'STORAGE')))",
        "CREATE INDEX ix_project_export_project_id ON project_export (project_id)",
        "CREATE TABLE project_export_file (
            export_id TEXT NOT NULL CONSTRAINT pk_project_export_file PRIMARY KEY,
            bytes BLOB NOT NULL,
            CONSTRAINT fk_project_export_file_export FOREIGN KEY (export_id) REFERENCES project_export (id) ON DELETE CASCADE)",
    ])
    .await
});
