//! `033`: the folders of a project that are parts of it, each with a name and a description. Their
//! own table, since a project has any number of them and each is edited on its own.

use super::{changeset, exec};

changeset!(ProjectComponent, "033-project-component", |manager| {
    exec(manager, &[
        "CREATE TABLE project_component (
            id TEXT NOT NULL CONSTRAINT pk_project_component PRIMARY KEY,
            project_id TEXT NOT NULL,
            name VARCHAR(200) NOT NULL,
            folder VARCHAR(1000) NOT NULL,
            description TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_project_component_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_project_component_project ON project_component (project_id, position)",
    ])
    .await
});
