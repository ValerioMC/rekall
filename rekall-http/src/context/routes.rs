use axum::Router;

use super::{context_size_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(context_size_controller::routes())
}
