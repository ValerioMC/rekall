use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rekall_common::RekallError;
use tracing::error;

use super::{Problem, framework};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    /// A refusal from a service, rendered as a problem detail.
    Domain(RekallError),
    /// The constraints `Validator` checks on a request body, one entry per broken field.
    Validation(Vec<(String, String)>),
    /// A body that could not be parsed into the request type.
    Unreadable(String),
    /// A failure of routing or extraction, before a handler runs.
    Framework(StatusCode),
}

impl From<RekallError> for ApiError {
    fn from(error: RekallError) -> Self {
        Self::Domain(error)
    }
}

impl ApiError {
    fn problem(&self) -> Option<Problem> {
        let (status, detail) = match self {
            Self::Domain(error) => match error {
                RekallError::NotFound(m) | RekallError::UnknownAnchor(m) => (StatusCode::NOT_FOUND, m.clone()),
                RekallError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
                RekallError::AmbiguousAnchor { message, .. } => (StatusCode::CONFLICT, message.clone()),
                RekallError::Integrity(cause) => (StatusCode::CONFLICT, explain(cause)),
                RekallError::IllegalArgument(m) => (StatusCode::BAD_REQUEST, m.clone()),
                RekallError::Internal(m) => {
                    error!("Unhandled failure: {m}");
                    (StatusCode::INTERNAL_SERVER_ERROR, m.clone())
                }
            },
            Self::Validation(errors) => (
                StatusCode::BAD_REQUEST,
                errors.iter().map(|(field, message)| format!("{field}: {message}")).collect::<Vec<_>>().join("; "),
            ),
            Self::Unreadable(message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::Framework(_) => return None,
        };
        Some(Problem { status, detail })
    }
}

/// The fallback for a write the services did not check first: the services refuse known clashes
/// with their own message before writing, so reaching this means a rule they do not know.
fn explain(cause: &str) -> String {
    if cause.contains("FOREIGN KEY") {
        return "This change points at a record that is not there, or removes one something still points at.".into();
    }
    format!("This change violates a database constraint: {cause}")
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.problem() {
            Some(problem) => {
                let mut response = problem.status.into_response();
                response.extensions_mut().insert(problem);
                response
            }
            None => {
                let Self::Framework(status) = self else { unreachable!() };
                framework(status)
            }
        }
    }
}
