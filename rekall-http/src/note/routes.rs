use axum::Router;

use super::{controller::document_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(document_controller::routes())
}
