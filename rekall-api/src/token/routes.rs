use axum::Router;

use super::{token_usage_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(token_usage_controller::routes())
}
