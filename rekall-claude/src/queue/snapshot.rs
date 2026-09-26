use rekall_common::Instant;
use rekall_model::RunQueueState;

/// Settings and state as the runner reads them at a decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub state: RunQueueState,
    pub start_at: Option<Instant>,
    pub hold_until: Option<Instant>,
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
}
