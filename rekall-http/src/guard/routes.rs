use axum::Router;

use super::controller::guard_context_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(guard_context_controller::routes())
}
