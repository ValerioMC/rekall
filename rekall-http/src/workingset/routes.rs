use axum::Router;

use super::controller::working_set_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(working_set_controller::routes())
}
