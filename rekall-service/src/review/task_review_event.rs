use rekall_common::Id;
use serde::Serialize;

use super::TaskReviewView;

/// Emitted whenever a stepless task's review line moves, over the same feed the steps use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReviewEvent {
    pub task_id: Id,
    pub review: TaskReviewView,
}
