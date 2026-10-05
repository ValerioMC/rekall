use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// `201 Created` with the JSON body.
pub fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, axum::Json(body)).into_response()
}

/// `204 No Content`.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}
