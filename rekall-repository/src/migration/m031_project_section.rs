//! `031`: a project's own titled markdown sections, read by the console alone. They sit in a table
//! of their own, not in `document`, because a note is reached through tasks and loads into a
//! context, and a section does neither.

use super::{changeset, exec};

changeset!(ProjectSection, "031-project-section", |manager| {
    exec(manager, &[
        "CREATE TABLE project_section (
            id TEXT NOT NULL CONSTRAINT pk_project_section PRIMARY KEY,
            project_id TEXT NOT NULL,
            title VARCHAR(200) NOT NULL,
            body_markdown TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_project_section_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_project_section_project ON project_section (project_id, position)",
    ])
    .await
});
