use rekall_console::adherence::ProjectAdherenceView;
use serde::Serialize;

use crate::adherence::TaskSentBackResponse;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAdherenceResponse {
    pub steps_total: usize,
    pub steps_done: usize,
    pub steps_claimed: usize,
    pub accepted_first_time: usize,
    pub sent_back: usize,
    pub passes_total: usize,
    pub first_time_share: Option<f64>,
    pub deviation_notes: usize,
    pub most_sent_back: Vec<TaskSentBackResponse>,
}

impl From<ProjectAdherenceView> for ProjectAdherenceResponse {
    fn from(view: ProjectAdherenceView) -> Self {
        Self {
            steps_total: view.counts.steps_total,
            steps_done: view.counts.steps_done,
            steps_claimed: view.counts.steps_claimed,
            accepted_first_time: view.counts.accepted_first_time,
            sent_back: view.counts.sent_back,
            passes_total: view.counts.passes_total,
            first_time_share: view.counts.first_time_share(),
            deviation_notes: view.deviation_notes,
            most_sent_back: view.most_sent_back.into_iter().map(TaskSentBackResponse::from).collect(),
        }
    }
}
