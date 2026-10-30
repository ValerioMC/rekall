use axum::Router;

use super::controller::telemetry_controller;
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(telemetry_controller::routes())
}
