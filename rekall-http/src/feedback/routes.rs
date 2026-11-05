use axum::Router;

use super::controller::feedback_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(feedback_controller::routes())
}
