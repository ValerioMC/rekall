use rekall_common::Id;

use super::GraphRouteCommand;

/// One step of a graph being written. `key` is the caller's name for it, unique in the request,
/// which the routes point at; it is not stored. `flow_x` and `flow_y` come together or not at all.
#[derive(Clone, Debug, Default)]
pub struct GraphStepCommand {
    pub key: Option<String>,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub connection_id: Option<Id>,
    pub max_visits: Option<i32>,
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    pub routes: Option<Vec<GraphRouteCommand>>,
}
