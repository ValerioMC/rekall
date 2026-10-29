use axum::Router;

use crate::{connection, resources, usage, ClaudeState};

/// The usage and resource routes, plus the terminals' credential helper, over one shared state.
pub fn router(state: ClaudeState) -> Router {
    Router::new()
        .merge(usage::routes())
        .merge(resources::routes())
        .route_layer(axum::middleware::from_fn(rekall_web::extract::uuid_path_params))
        .merge(connection::routes())
        .with_state(state)
}
