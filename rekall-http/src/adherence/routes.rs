use axum::Router;

use super::controller::adherence_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(adherence_controller::routes())
}
