use axum::Router;

use super::{controller::step_flow_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(step_flow_controller::routes())
}
