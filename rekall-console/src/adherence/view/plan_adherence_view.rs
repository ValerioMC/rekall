use crate::adherence::{AdherenceCounts, StepVisitsView};

/// One task's adherence. `deviation_steps` holds the step each deviation of the task refers to, in step order;
/// `visits` is empty unless the task ran as a flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanAdherenceView {
    pub counts: AdherenceCounts,
    pub deviation_steps: Vec<usize>,
    pub visits: Vec<StepVisitsView>,
}
