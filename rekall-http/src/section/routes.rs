use axum::Router;

use super::controller::{project_section_controller, section_context_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(project_section_controller::routes())
        .merge(section_context_controller::routes())
}
