//! What Spring MVC did with a method a path does not take. An `OPTIONS` request is answered
//! `200` with the methods the path takes and `OPTIONS` itself; any other method is a `405` whose
//! `Allow` lists the declared methods, comma-and-space separated and without the implicit `HEAD`.
//! Axum answers both with a `405` carrying its own `Allow`, which this rewrites.

use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub async fn spring_methods(request: Request, next: Next) -> Response {
    let options = request.method() == Method::OPTIONS;
    let mut response = next.run(request).await;
    if response.status() != StatusCode::METHOD_NOT_ALLOWED {
        return response;
    }
    let Some(allowed) = response.headers().get(header::ALLOW).and_then(|a| a.to_str().ok()).map(str::to_string) else {
        return response;
    };
    let methods: Vec<&str> = allowed.split(',').map(str::trim).filter(|m| !m.is_empty()).collect();
    if options {
        let mut answer = StatusCode::OK.into_response();
        let with_options = format!("{},OPTIONS", methods.join(","));
        if let Ok(value) = HeaderValue::from_str(&with_options) {
            answer.headers_mut().insert(header::ALLOW, value);
        }
        answer.headers_mut().insert("accept-patch", HeaderValue::from_static(""));
        return answer;
    }
    let declared: Vec<&str> = methods.into_iter().filter(|m| *m != "HEAD").collect();
    if let Ok(value) = HeaderValue::from_str(&declared.join(", ")) {
        response.headers_mut().insert(header::ALLOW, value);
    }
    response
}
