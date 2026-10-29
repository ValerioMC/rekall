use rekall_kernel::Id;
use rekall_console::graph::GraphStepView;
use serde::Serialize;

use crate::graph::GraphRouteResponse;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphStepResponse {
    pub id: Id,
    pub title: String,
    pub detail: Option<String>,
    pub connection_id: Option<Id>,
    pub max_visits: Option<i32>,
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    pub routes: Vec<GraphRouteResponse>,
}

impl From<GraphStepView> for GraphStepResponse {
    fn from(view: GraphStepView) -> Self {
        Self {
            id: view.id,
            title: view.title,
            detail: view.detail,
            connection_id: view.connection_id,
            max_visits: view.max_visits,
            flow_x: view.flow_x,
            flow_y: view.flow_y,
            routes: view.routes.into_iter().map(GraphRouteResponse::from).collect(),
        }
    }
}
