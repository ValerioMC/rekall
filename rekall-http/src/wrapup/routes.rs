use axum::Router;

use super::{controller::wrapup_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(wrapup_controller::routes())
}
