use crate::adherence::{AdherenceCounts, TaskSentBackView};

/// The task numbers summed over a project. `most_sent_back` lists the tasks with the most steps
/// sent back, never more than five, and none that had no step sent back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAdherenceView {
    pub counts: AdherenceCounts,
    pub deviations: usize,
    pub most_sent_back: Vec<TaskSentBackView>,
}
