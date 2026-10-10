use rekall_common::Id;
use serde::Serialize;

use super::TokenTallyView;

/// A task's tallies after a transcript read changed them, all of them, so the console replaces
/// what it held for that task.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageStreamEvent {
    pub task_id: Id,
    pub tallies: Vec<TokenTallyView>,
}
