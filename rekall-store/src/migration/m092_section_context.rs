//! `092`: a project's sections are kept in contexts, titled groups a person arranges. Every
//! project that already has sections gets one context, "General", and they all go into it.

use super::{changeset, exec};

const NEW_ID: &str = "lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6)))";

changeset!(SectionContext, "092-section-context", |manager| {
    let general_context_per_project = format!(
        "INSERT INTO section_context (id, project_id, title, position, created_at, updated_at)
            SELECT {NEW_ID}, project_id, 'General', 0, MIN(created_at), MAX(updated_at) FROM project_section GROUP BY project_id"
    );
    exec(manager, &[
        "CREATE TABLE section_context (
            id TEXT NOT NULL CONSTRAINT pk_section_context PRIMARY KEY,
            project_id TEXT NOT NULL,
            title VARCHAR(200) NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CONSTRAINT fk_section_context_project FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE CASCADE)",
        "CREATE INDEX ix_section_context_project ON section_context (project_id, position)",
        &general_context_per_project,
        "ALTER TABLE project_section ADD COLUMN context_id TEXT REFERENCES section_context (id)",
        "UPDATE project_section SET context_id = (SELECT id FROM section_context WHERE section_context.project_id = project_section.project_id)",
        "CREATE INDEX ix_project_section_context ON project_section (context_id, position)",
    ])
    .await
});
