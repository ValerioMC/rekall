//! `084`: a note belongs to one task. `document` gains `task_id` and `position`; `document_task`,
//! `scope_company_id`, `scope_project_id` and the scope check of `078` are dropped.
//!
//! Data rules, since this rewrites user data. A note on several tasks becomes one copy per task:
//! the first task it was attached to keeps the original id, the others get a new id and the same
//! text, each keeping the position the note had on its task. A note on no task is deleted: with
//! no task to open it from, nothing can show it any more.

use super::{changeset, exec};

const NEW_ID: &str = "lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)), 2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6)))";

changeset!(NoteOwnedByTask, "084-note-owned-by-task", |manager| {
    let copy_one_note_per_task = format!(
        "INSERT INTO document_owned (id, task_id, position, title, kind, body_markdown, source_path, created_at, updated_at, context_mode)
         SELECT CASE WHEN l.attached = 1 THEN d.id ELSE {NEW_ID} END, l.task_id, l.position, d.title, d.kind, d.body_markdown, d.source_path, d.created_at, d.updated_at, d.context_mode
         FROM document d
         JOIN (SELECT document_id, task_id, position, ROW_NUMBER() OVER (PARTITION BY document_id ORDER BY rowid) AS attached FROM document_task) l
           ON l.document_id = d.id"
    );
    exec(manager, &[
        "CREATE TABLE document_owned (id TEXT NOT NULL CONSTRAINT pk_document PRIMARY KEY,
         task_id TEXT NOT NULL CONSTRAINT fk_document_task REFERENCES task (id) ON DELETE CASCADE,
         position INTEGER NOT NULL,
         title VARCHAR(255) NOT NULL,
         kind VARCHAR(40) NOT NULL,
         body_markdown VARCHAR(100000) NOT NULL,
         source_path VARCHAR(500),
         created_at TEXT NOT NULL,
         updated_at TEXT NOT NULL,
         context_mode VARCHAR(20) NOT NULL DEFAULT 'FULL')",
        &copy_one_note_per_task,
        "DROP TABLE document_task",
        "DROP TABLE document",
        "ALTER TABLE document_owned RENAME TO document",
        "CREATE INDEX idx_document_task ON document (task_id, position)",
    ])
    .await
});
