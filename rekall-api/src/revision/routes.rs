use axum::Router;

use super::{task_revision_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(task_revision_controller::routes())
}
