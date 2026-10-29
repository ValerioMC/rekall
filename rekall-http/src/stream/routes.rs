use axum::Router;

use super::{controller::step_event_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(step_event_controller::routes())
}
