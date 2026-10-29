use axum::Router;

use super::{controller::directory_listing_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(directory_listing_controller::routes())
}
