use rekall_common::Id;
use rekall_model::graph_route;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRouteResponse {
    pub outcome: String,
    /// The step it leads to; `None` ends the flow.
    pub to_step_id: Option<Id>,
}

impl GraphRouteResponse {
    pub fn of(route: &graph_route::Model) -> Self {
        Self { outcome: route.outcome.clone(), to_step_id: route.to_step_id }
    }
}
