use axum::Router;

use super::{skill_assignment_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(skill_assignment_controller::routes())
}
