use axum::Router;

use super::controller::{chat_controller, chat_events_controller};
use crate::ApiState;

pub(crate) fn routes() -> Router<ApiState> {
    chat_events_controller::routes().merge(chat_controller::routes())
}

/// The permission prompt is named by the id `claude` gave it, which is not promised to be a UUID,
/// so its route sits outside the layer that reads every path variable as one.
pub(crate) fn open_routes() -> Router<ApiState> {
    chat_controller::permission_routes()
}
