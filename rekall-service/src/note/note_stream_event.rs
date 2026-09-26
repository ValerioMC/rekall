use rekall_common::Id;
use serde::Serialize;

/// Emitted when a session writes or rewrites a note on a task, so an open console can pick it up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteStreamEvent {
    pub task_id: Id,
    pub document_id: Id,
}
