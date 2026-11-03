use rekall_console::adherence::PlanAdherenceView;
use serde::Serialize;

use crate::adherence::StepVisitsResponse;

/// `firstTimeShare` is a fraction from 0 to 1, absent until a step has been accepted.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanAdherenceResponse {
    pub steps_total: usize,
    pub steps_done: usize,
    pub steps_claimed: usize,
    pub accepted_first_time: usize,
    pub sent_back: usize,
    pub passes_total: usize,
    pub first_time_share: Option<f64>,
    pub deviation_notes: usize,
    pub deviation_steps: Vec<usize>,
    pub visits: Vec<StepVisitsResponse>,
}

impl From<PlanAdherenceView> for PlanAdherenceResponse {
    fn from(view: PlanAdherenceView) -> Self {
        Self {
            steps_total: view.counts.steps_total,
            steps_done: view.counts.steps_done,
            steps_claimed: view.counts.steps_claimed,
            accepted_first_time: view.counts.accepted_first_time,
            sent_back: view.counts.sent_back,
            passes_total: view.counts.passes_total,
            first_time_share: view.counts.first_time_share(),
            deviation_notes: view.deviation_steps.len(),
            deviation_steps: view.deviation_steps,
            visits: view.visits.into_iter().map(StepVisitsResponse::from).collect(),
        }
    }
}
