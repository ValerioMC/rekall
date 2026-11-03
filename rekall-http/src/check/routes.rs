use axum::Router;

use super::controller::check_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(check_controller::routes())
}
