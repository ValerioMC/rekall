use super::TaskStepView;

/// A draft a session proposed, and how many drafts the task now holds.
#[derive(Clone, Debug)]
pub struct Proposed {
    pub step: TaskStepView,
    pub draft_count: usize,
}
