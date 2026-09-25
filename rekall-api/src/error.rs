//! `RestExceptionHandler`: every failure a handler returns becomes an RFC 7807 problem, with the
//! status the Java handler chose for its exception class and the request path as `instance`, the
//! way Spring filled it in. Failures Spring raised before a controller ran (a missing query
//! parameter, a path nothing answers, a method nothing takes) keep the shape Spring Boot's error
//! controller gave them instead: `timestamp`, `status`, `error`, `path`.

use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use rekall_common::RekallError;
use serde_json::json;
use tracing::error;

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

/// The problem a failed request carries until the middleware knows its path.
#[derive(Clone, Debug)]
pub struct Problem {
    pub status: StatusCode,
    pub detail: String,
}

/// Marks a framework error the middleware renders in Spring Boot's error-controller shape.
#[derive(Clone, Copy, Debug)]
pub struct FrameworkError(pub StatusCode);

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

/// A framework-level failure, rendered by `render_errors` once the path is known.
pub fn framework(status: StatusCode) -> Response {
    let mut response = status.into_response();
    response.extensions_mut().insert(FrameworkError(status));
    response
}

fn reason(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("Error")
}

/// Writes the body of every failed response, now that the request path is known.
pub async fn render_errors(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let wants_html = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|accept| accept.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"));
    let mut response = next.run(request).await;
    if let Some(problem) = response.extensions_mut().remove::<Problem>() {
        // What Spring wrote for a `ProblemDetail`: its properties in alphabetical order, and no
        // `type` when it is the default `about:blank`.
        let body = json!({
            "detail": problem.detail,
            "instance": path,
            "status": problem.status.as_u16(),
            "title": reason(problem.status),
        });
        let mut rendered = (problem.status, body.to_string()).into_response();
        rendered
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
        return rendered;
    }
    if let Some(FrameworkError(status)) = response.extensions_mut().remove::<FrameworkError>() {
        if wants_html {
            return whitelabel(status, response.headers().clone());
        }
        // `java.util.Date`, as Jackson writes it: milliseconds, always three digits.
        let body = json!({
            "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "status": status.as_u16(),
            "error": reason(status),
            "path": path,
        });
        let mut rendered = (status, body.to_string()).into_response();
        if let Some(allow) = response.headers().get(header::ALLOW) {
            rendered.headers_mut().insert(header::ALLOW, allow.clone());
        }
        rendered
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        return rendered;
    }
    response
}

/// `BasicErrorController.errorHtml`: what a browser asking for HTML got, the Whitelabel page.
/// Spring printed the time as `java.util.Date` does, in the JVM's zone; this prints it in UTC.
fn whitelabel(status: StatusCode, headers: axum::http::HeaderMap) -> Response {
    let created = chrono::Utc::now().format("%a %b %d %H:%M:%S UTC %Y");
    let body = format!(
        "<html><body><h1>Whitelabel Error Page</h1><p>This application has no explicit mapping for /error, so you are \
         seeing this as a fallback.</p><div id='created'>{created}</div><div>There was an unexpected error \
         (type={}, status={}).</div></body></html>",
        reason(status),
        status.as_u16()
    );
    let mut rendered = (status, body).into_response();
    if let Some(allow) = headers.get(header::ALLOW) {
        rendered.headers_mut().insert(header::ALLOW, allow.clone());
    }
    rendered.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html;charset=UTF-8"));
    rendered
}
