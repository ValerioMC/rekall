use rekall_common::{Id, Instant};
use rekall_store::step_visit;
use serde::Serialize;

/// One entry of the flow into a step; `ordinal` is its move number across the whole task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepVisitView {
    pub id: Id,
    pub step_id: Id,
    pub ordinal: i32,
    pub entered_at: Instant,
    pub left_at: Option<Instant>,
    pub outcome: Option<String>,
    pub report: Option<String>,
}

impl StepVisitView {
    pub fn of(visit: &step_visit::Model) -> Self {
        Self {
            id: visit.id,
            step_id: visit.step_id,
            ordinal: visit.ordinal,
            entered_at: visit.entered_at,
            left_at: visit.left_at,
            outcome: visit.outcome.clone(),
            report: visit.report.clone(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.left_at.is_none()
    }
}
