use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rekall_common::RekallError;
use tracing::error;

use super::{Problem, framework};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    /// Something the exception handler mapped to a problem detail.
    Domain(RekallError),
    /// `MethodArgumentNotValidException`: the `@Valid` constraints of a request body.
    Validation(Vec<(String, String)>),
    /// `HttpMessageNotReadableException`: a body Jackson could not read.
    Unreadable(String),
    /// An error Spring answered on its own, before or around the controller.
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

/// `RestExceptionHandler.explain`: a constraint named in the cause gets a sentence of its own.
fn explain(cause: &str) -> String {
    if cause.contains("still referenced") {
        return "Something still references this record. Delete or repoint those records first.".into();
    }
    if cause.contains("UQ_PROJECT_COMPANY_LABEL") {
        return "Another project in this company already uses that label. An anchor has to name one record.".into();
    }
    if cause.contains("UQ_TASK_PROJECT_LABEL") {
        return "Another task on this project already uses that label. An anchor has to name one record.".into();
    }
    if cause.contains("UQ_COMPANY_NAME") {
        return "A company with that name already exists.".into();
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
