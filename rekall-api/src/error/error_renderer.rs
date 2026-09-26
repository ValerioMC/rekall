use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use super::{FrameworkError, Problem};

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
