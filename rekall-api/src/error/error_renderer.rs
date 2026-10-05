use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use super::{FrameworkError, Problem};

fn reason(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("Error")
}

/// Writes the body of every failed response, now that the request path is known: a domain
/// refusal as RFC 9457 problem details, a routing or extraction failure as a small JSON status.
pub async fn render_errors(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    if let Some(problem) = response.extensions_mut().remove::<Problem>() {
        return problem_details(problem, &path);
    }
    if let Some(FrameworkError(status)) = response.extensions_mut().remove::<FrameworkError>() {
        return framework_status(status, &path, response.headers());
    }
    response
}

fn problem_details(problem: Problem, path: &str) -> Response {
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
    rendered
}

/// Keeps `Allow`, so a 405 still says which methods the path accepts.
fn framework_status(status: StatusCode, path: &str, original: &HeaderMap) -> Response {
    let body = json!({
        "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "status": status.as_u16(),
        "error": reason(status),
        "path": path,
    });
    let mut rendered = (status, body.to_string()).into_response();
    if let Some(allow) = original.get(header::ALLOW) {
        rendered.headers_mut().insert(header::ALLOW, allow.clone());
    }
    rendered
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    rendered
}
