use axum::Router;

use super::{controller::socket_controller, controller::terminal_controller};
use crate::ApiState;

pub(crate) fn routes() -> Router<ApiState> {
    terminal_controller::routes()
}

/// The socket reads its own id: a bad one closes the socket rather than failing the handshake.
pub(crate) fn socket_routes() -> Router<ApiState> {
    socket_controller::routes()
}
