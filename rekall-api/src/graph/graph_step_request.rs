use rekall_kernel::Id;
use rekall_console::graph::{GraphRouteCommand, GraphStepCommand};
use serde::Deserialize;

use super::GraphRouteRequest;

/// One step of a graph being written. `key` is the caller's name for it, unique in the request,
/// which the routes point at; it is not stored. `flow_x` and `flow_y` come together or not at all.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GraphStepRequest {
    pub key: Option<String>,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub connection_id: Option<Id>,
    pub max_visits: Option<i32>,
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    pub routes: Option<Vec<GraphRouteRequest>>,
}

impl From<GraphStepRequest> for GraphStepCommand {
    fn from(request: GraphStepRequest) -> Self {
        Self {
            key: request.key,
            title: request.title,
            detail: request.detail,
            connection_id: request.connection_id,
            max_visits: request.max_visits,
            flow_x: request.flow_x,
            flow_y: request.flow_y,
            routes: request.routes.map(|routes| routes.into_iter().map(GraphRouteCommand::from).collect()),
        }
    }
}
