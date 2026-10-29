use axum::Router;

use super::{search_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(search_controller::routes())
}
