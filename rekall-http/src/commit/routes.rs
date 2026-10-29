use axum::Router;

use super::{controller::commit_reference_controller};
use crate::ApiState;

/// Every route of this domain.
pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(commit_reference_controller::routes())
}
