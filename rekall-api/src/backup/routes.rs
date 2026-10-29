use axum::Router;

use super::backup_controller;
use crate::ApiState;

/// Every route of this domain. A backup is addressed by its file name, so these sit outside the
/// layer that refuses a path parameter that is not a uuid.
pub fn routes() -> Router<ApiState> {
    Router::new().merge(backup_controller::routes())
}
