use axum::Router;

use super::controller::task_branch_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(task_branch_controller::routes())
}
