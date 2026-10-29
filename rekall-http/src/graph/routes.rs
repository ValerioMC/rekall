use axum::Router;

use super::{controller::deployment_graph_controller, controller::graph_attach_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(deployment_graph_controller::routes()).merge(graph_attach_controller::routes())
}
