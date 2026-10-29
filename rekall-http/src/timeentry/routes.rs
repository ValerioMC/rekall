use axum::Router;

use super::{time_entry_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(time_entry_controller::routes())
}
