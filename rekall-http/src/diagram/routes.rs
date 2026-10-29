use axum::Router;

use super::{controller::diagram_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(diagram_controller::routes())
}
