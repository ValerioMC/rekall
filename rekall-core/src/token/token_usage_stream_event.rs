use rekall_kernel::Id;
use serde::Serialize;

use super::{TokenDayView, TokenTallyView};

/// A task's tallies and days after a transcript read changed them, all of them, so the console
/// replaces what it held for that task.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageStreamEvent {
    pub task_id: Id,
    pub tallies: Vec<TokenTallyView>,
    pub days: Vec<TokenDayView>,
}

impl crate::StreamEvent for TokenUsageStreamEvent {
    const FRAME: &'static str = "token-usage";
}
