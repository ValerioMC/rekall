use rekall_common::Id;
use rekall_model::RevisionKind;
use serde::Serialize;

/// What a restore wrote back: which text of which task, and its new current body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoredRevision {
    pub task_id: Id,
    pub kind: RevisionKind,
    pub body_markdown: String,
}
