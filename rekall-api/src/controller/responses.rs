use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// `@ResponseStatus(HttpStatus.CREATED)` on a JSON answer.
pub fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, axum::Json(body)).into_response()
}

/// `@ResponseStatus(HttpStatus.NO_CONTENT)` on a void method.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}
