use rekall_kernel::{Id, Instant};
use rekall_store::context_fill;
use serde::Serialize;

use crate::chat::ContextFill;

/// The last fill of a task, or of one of its steps when `step_id` is set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextFillView {
    pub step_id: Option<Id>,
    pub used_tokens: u64,
    pub window_tokens: u64,
    pub percent: u8,
    pub recorded_at: Instant,
}

impl ContextFillView {
    pub fn of(row: &context_fill::Model) -> Self {
        let fill = ContextFill {
            used_tokens: u64::try_from(row.used_tokens).unwrap_or(0),
            window_tokens: u64::try_from(row.window_tokens).unwrap_or(0),
        };
        Self {
            step_id: row.step_id,
            used_tokens: fill.used_tokens,
            window_tokens: fill.window_tokens,
            percent: fill.percent(),
            recorded_at: row.recorded_at,
        }
    }
}
