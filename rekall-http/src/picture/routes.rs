use axum::Router;

use super::controller::picture_controller;
use crate::ApiState;

/// Every route of this domain. A picture is addressed by its path, so these sit outside the layer
/// that refuses a path parameter that is not a uuid.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(picture_controller::routes())
}
