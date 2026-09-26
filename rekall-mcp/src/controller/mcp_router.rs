use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde_json::{json, Value};

use crate::protocol::Request;

use super::McpController;

/// `POST /mcp`.
pub fn router(controller: McpController) -> Router {
    Router::new().route("/mcp", post(handle_http)).with_state(Arc::new(controller))
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

async fn handle_http(State(controller): State<Arc<McpController>>, headers: HeaderMap, body: Bytes) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value @ Value::Object(_)) => value,
        Ok(_) => return unreadable("Cannot deserialize value of type `dev.rekall.mcp.protocol.JsonRpc$Request`"),
        Err(e) if body.iter().all(u8::is_ascii_whitespace) => {
            let _ = e;
            return unreadable("Required request body is missing");
        }
        Err(e) => return unreadable(&e.to_string()),
    };
    let request = Request::from_value(&value);
    controller
        .handle(
            header(&headers, "MCP-Protocol-Version"),
            header(&headers, "Mcp-Method"),
            header(&headers, "Mcp-Name"),
            &request,
        )
        .await
        .into_response()
}

/// A body that is not a JSON object is the 400 problem `HttpMessageNotReadableException` gave.
fn unreadable(detail: &str) -> Response {
    let body = json!({
        "detail": detail,
        "instance": "/mcp",
        "status": 400,
        "title": "Bad Request",
    });
    let mut response = (StatusCode::BAD_REQUEST, body.to_string()).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
    response
}
