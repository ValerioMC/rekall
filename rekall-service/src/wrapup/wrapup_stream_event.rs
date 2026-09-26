use rekall_common::Id;
use serde::Serialize;

use super::WrapupView;

/// Emitted when a task's wrapup is written or deleted. On a delete `wrapup` is null.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupStreamEvent {
    pub task_id: Id,
    pub wrapup: Option<WrapupView>,
    pub deleted: bool,
}
