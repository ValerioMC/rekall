use axum::Router;

use crate::{queue, socket, terminal, usage, ClaudeState};

/// The terminal, usage and queue routes, plus the terminal socket, over one shared state.
pub fn router(state: ClaudeState) -> Router {
    Router::new()
        .merge(terminal::routes())
        .merge(usage::routes())
        .merge(queue::routes())
        .route_layer(axum::middleware::from_fn(rekall_api::extract::uuid_path_params))
        // The socket reads its own id: a bad one closes the socket, as the handshake handler did.
        .merge(socket::routes())
        .with_state(state)
}
