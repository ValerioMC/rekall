use axum::Router;

use super::controller::quick_task_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(quick_task_controller::routes())
}
