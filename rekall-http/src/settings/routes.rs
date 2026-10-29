use axum::Router;

use super::controller::settings_controller;
use crate::ApiState;

/// Every route of this domain. A database id that is not registered is a 404 whatever its form, so
/// these sit outside the layer that refuses a path parameter that is not a uuid.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(settings_controller::routes())
}
