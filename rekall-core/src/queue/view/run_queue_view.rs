use rekall_kernel::Instant;
use rekall_store::{run_queue, RunQueueState, SessionKind};
use serde::Serialize;

use crate::queue::RunQueueItemView;

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
    pub session_kind: SessionKind,
    pub parallel_sessions: i32,
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
            session_kind: queue.session_kind,
            parallel_sessions: queue.parallel_sessions,
            hold_until: queue.hold_until,
            hold_reason: queue.hold_reason.clone(),
            items,
            updated_at: queue.updated_at,
        }
    }
}
