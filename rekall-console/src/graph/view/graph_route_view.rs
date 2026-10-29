use rekall_kernel::Id;
use rekall_store::graph_route;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphRouteView {
    pub outcome: String,
    /// The step it leads to; `None` ends the flow.
    pub to_step_id: Option<Id>,
}

impl GraphRouteView {
    pub fn of(route: &graph_route::Model) -> Self {
        Self { outcome: route.outcome.clone(), to_step_id: route.to_step_id }
    }
}
