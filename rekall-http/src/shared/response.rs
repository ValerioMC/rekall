use axum::http::{header, HeaderValue, StatusCode};
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

/// A rendered PNG; `versioned` means its URL names one image for good, so the browser keeps it.
pub fn png(bytes: Vec<u8>, versioned: bool) -> Response {
    let cache = if versioned { "public, max-age=31536000, immutable" } else { "no-cache" };
    ([(header::CONTENT_TYPE, HeaderValue::from_static("image/png")), (header::CACHE_CONTROL, HeaderValue::from_static(cache))], bytes).into_response()
}
