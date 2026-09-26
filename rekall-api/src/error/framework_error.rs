use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Marks a framework error the middleware renders in Spring Boot's error-controller shape.
#[derive(Clone, Copy, Debug)]
pub struct FrameworkError(pub StatusCode);

/// A framework-level failure, rendered by `render_errors` once the path is known.
pub fn framework(status: StatusCode) -> Response {
    let mut response = status.into_response();
    response.extensions_mut().insert(FrameworkError(status));
    response
}
