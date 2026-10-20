/// A route a plan proposes out of one of its steps: the outcome it leaves by and the step of the
/// plan it leads to, by zero-based position, or `None` for the end of the flow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedRoute {
    pub outcome: String,
    pub to: Option<usize>,
}
