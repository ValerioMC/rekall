use rekall_common::Id;
use rekall_console::graph::GraphRouteView;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRouteResponse {
    pub outcome: String,
    /// The step it leads to; `None` ends the flow.
    pub to_step_id: Option<Id>,
}

impl From<GraphRouteView> for GraphRouteResponse {
    fn from(view: GraphRouteView) -> Self {
        Self {
            outcome: view.outcome,
            to_step_id: view.to_step_id,
        }
    }
}
