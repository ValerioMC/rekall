use rekall_common::Id;
use serde::Serialize;

use super::CommitReferenceView;

/// Emitted when a commit is logged against a task or step, whichever path logged it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceStreamEvent {
    pub task_id: Id,
    pub reference: CommitReferenceView,
}
