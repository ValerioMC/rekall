//! `030`: a note is owned by a company or a project, or by nothing (global). Each existing note
//! takes the narrowest scope its tasks share, so nothing a session reads changes.

use super::{changeset, exec};

changeset!(NoteScope, "030-note-scope", |manager| {
    exec(
        manager,
        &[
            "ALTER TABLE document ADD COLUMN scope_company_id TEXT \
             CONSTRAINT fk_document_scope_company REFERENCES company (id) ON DELETE CASCADE",
            "ALTER TABLE document ADD COLUMN scope_project_id TEXT \
             CONSTRAINT fk_document_scope_project REFERENCES project (id) ON DELETE CASCADE",
            "UPDATE document SET scope_project_id = (
                 SELECT MIN(t.project_id) FROM document_task dt JOIN task t ON t.id = dt.task_id
                 WHERE dt.document_id = document.id)
             WHERE (SELECT COUNT(DISTINCT t.project_id) FROM document_task dt JOIN task t ON t.id = dt.task_id
                    WHERE dt.document_id = document.id) = 1",
            "UPDATE document SET scope_company_id = (
                 SELECT MIN(p.company_id) FROM document_task dt JOIN task t ON t.id = dt.task_id
                 JOIN project p ON p.id = t.project_id WHERE dt.document_id = document.id)
             WHERE scope_project_id IS NULL
               AND (SELECT COUNT(DISTINCT p.company_id) FROM document_task dt JOIN task t ON t.id = dt.task_id
                    JOIN project p ON p.id = t.project_id WHERE dt.document_id = document.id) = 1",
            "CREATE INDEX idx_document_scope_company ON document (scope_company_id)",
            "CREATE INDEX idx_document_scope_project ON document (scope_project_id)",
        ],
    )
    .await
});
