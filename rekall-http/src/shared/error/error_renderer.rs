use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use super::{FrameworkError, Problem};

/// Writes the body of every failed response as RFC 9457 problem details, now that the request
/// path is known. A routing or extraction failure carries its status's reason as the detail.
pub async fn render_errors(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    if let Some(problem) = response.extensions_mut().remove::<Problem>() {
        return problem_details(problem, &path, None);
    }
    if let Some(FrameworkError(status)) = response.extensions_mut().remove::<FrameworkError>() {
        let problem = Problem { status, detail: reason(status).to_string() };
        return problem_details(problem, &path, Some(response.headers()));
    }
    response
}

/// `original`'s `Allow` is kept, so a 405 still says which methods the path accepts.
fn problem_details(problem: Problem, path: &str, original: Option<&HeaderMap>) -> Response {
    let body = json!({
        "detail": problem.detail,
        "instance": path,
        "status": problem.status.as_u16(),
        "title": reason(problem.status),
    });
    let mut rendered = (problem.status, body.to_string()).into_response();
    if let Some(allow) = original.and_then(|headers| headers.get(header::ALLOW)) {
        rendered.headers_mut().insert(header::ALLOW, allow.clone());
    }
    rendered
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
    rendered
}

fn reason(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("Error")
}
