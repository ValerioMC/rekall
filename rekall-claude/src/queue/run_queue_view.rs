use rekall_common::Instant;
use rekall_model::{run_queue, RunQueueState};
use serde::Serialize;

use super::RunQueueItemView;

/// The run queue as the console draws it: its state and settings, and every item in order. The
/// same shape answers every call and rides the `run-queue` event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueueView {
    pub state: RunQueueState,
    pub start_at: Option<Instant>,
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub hold_until: Option<Instant>,
    pub hold_reason: Option<String>,
    pub items: Vec<RunQueueItemView>,
    pub updated_at: Instant,
}

impl RunQueueView {
    pub fn of(queue: &run_queue::Model, items: Vec<RunQueueItemView>) -> Self {
        Self {
            state: queue.state,
            start_at: queue.start_at,
            ceiling_percent: queue.ceiling_percent,
            skip_permissions: queue.skip_permissions,
            model: queue.model.clone(),
            effort: queue.effort.clone(),
            hold_until: queue.hold_until,
            hold_reason: queue.hold_reason.clone(),
            items,
            updated_at: queue.updated_at,
        }
    }
}
