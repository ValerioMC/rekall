//! `051`: a project may bind one connection several times, once per repository or namespace, so a
//! binding carries its own label, unique in the project, and starts as its connection's.

use super::{changeset, exec};

changeset!(BindingLabel, "051-binding-label", |manager| {
    exec(manager, &[
        "CREATE TABLE project_connection_labelled (
            id TEXT NOT NULL CONSTRAINT pk_project_connection PRIMARY KEY,
            project_id TEXT NOT NULL,
            connection_id TEXT NOT NULL,
            label VARCHAR(120) NOT NULL,
            target VARCHAR(500),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_project_connection_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE,
            CONSTRAINT fk_project_connection_connection FOREIGN KEY (connection_id) REFERENCES external_connection (id) ON DELETE CASCADE,
            CONSTRAINT uq_project_connection_label UNIQUE (project_id, label))",
        "INSERT INTO project_connection_labelled (id, project_id, connection_id, label, target, created_at, updated_at)
            SELECT binding.id, binding.project_id, binding.connection_id, connection.label, binding.target, binding.created_at, binding.updated_at
            FROM project_connection binding JOIN external_connection connection ON connection.id = binding.connection_id",
        "DROP INDEX ix_project_connection_connection",
        "DROP TABLE project_connection",
        "ALTER TABLE project_connection_labelled RENAME TO project_connection",
        "CREATE INDEX ix_project_connection_connection ON project_connection (connection_id)",
    ])
    .await
});
