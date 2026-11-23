use rekall_core::decision::DecisionView;

/// A task's decisions, and the text its context carries for them: what the console shows as what a session reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDecisionsView {
    pub decisions: Vec<DecisionView>,
    pub session_block: String,
}
