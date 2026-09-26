use rekall_common::{Id, Instant};
use rekall_model::task_revision;
use rekall_model::{RevisionKind, WrapupAuthor};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRevisionView {
    pub id: Id,
    pub task_id: Id,
    pub kind: RevisionKind,
    pub body_markdown: String,
    pub written_by: Option<WrapupAuthor>,
    pub written_at: Option<Instant>,
    pub replaced_at: Instant,
}

impl TaskRevisionView {
    pub fn of(revision: &task_revision::Model) -> Self {
        Self {
            id: revision.id,
            task_id: revision.task_id,
            kind: revision.kind,
            body_markdown: revision.body_markdown.clone(),
            written_by: revision.written_by,
            written_at: revision.written_at,
            replaced_at: revision.created_at,
        }
    }
}
