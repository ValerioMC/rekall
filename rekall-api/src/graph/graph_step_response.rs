use rekall_common::Id;
use rekall_model::{graph_route, graph_step};
use serde::Serialize;

use super::GraphRouteResponse;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphStepResponse {
    pub id: Id,
    pub title: String,
    pub detail: Option<String>,
    pub connection_id: Option<Id>,
    pub max_visits: Option<i32>,
    pub routes: Vec<GraphRouteResponse>,
}

impl GraphStepResponse {
    /// The step with the routes that leave it, picked out of the whole graph's.
    pub fn of(step: &graph_step::Model, routes: &[graph_route::Model]) -> Self {
        Self {
            id: step.id,
            title: step.title.clone(),
            detail: step.body_markdown.clone(),
            connection_id: step.connection_id,
            max_visits: step.max_visits,
            routes: routes.iter().filter(|route| route.from_step_id == step.id).map(GraphRouteResponse::of).collect(),
        }
    }
}
