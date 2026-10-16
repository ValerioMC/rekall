use axum::Router;

use super::{project_section_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(project_section_controller::routes())
}
