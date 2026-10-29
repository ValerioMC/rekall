use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Marks a routing or extraction failure for `render_errors` to give a problem body.
#[derive(Clone, Copy, Debug)]
pub struct FrameworkError(pub StatusCode);

/// A framework-level failure, rendered by `render_errors` once the path is known.
pub fn framework(status: StatusCode) -> Response {
    let mut response = status.into_response();
    response.extensions_mut().insert(FrameworkError(status));
    response
}
