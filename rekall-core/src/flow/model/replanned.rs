use crate::flow::ProposedFlow;

/// What a replan did to a task: the drafts and open steps it removed, the plan it laid, and the
/// started or finished steps that stayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Replanned {
    pub removed: usize,
    pub laid: ProposedFlow,
    pub kept: usize,
}
