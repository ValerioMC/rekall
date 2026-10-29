use axum::http::StatusCode;

/// The problem a failed request carries until the middleware knows its path.
#[derive(Clone, Debug)]
pub struct Problem {
    pub status: StatusCode,
    pub detail: String,
}
