use axum::Router;

use super::version_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(version_controller::routes())
}
