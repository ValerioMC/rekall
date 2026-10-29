use crate::step::TaskStepView;

/// What a session's move did: the step it moved, and on a flow, where a claim sent the work.
#[derive(Clone, Debug)]
pub struct FlowMove {
    pub step: TaskStepView,
    pub route: Option<FlowRoute>,
}

/// Where a claim on a flow led.
#[derive(Clone, Debug)]
pub enum FlowRoute {
    /// The route `outcome` names, and the step it opened: the one to start next.
    Followed { outcome: String, next: Box<TaskStepView> },
    /// The route `outcome` leads to the end of the flow, or the step has no routes out.
    Ended { outcome: Option<String> },
}
