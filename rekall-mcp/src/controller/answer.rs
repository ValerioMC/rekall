use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::protocol::Response as RpcResponse;

/// What a call came to: a status and a JSON-RPC body, or an accepted notification with none.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // One per request, returned straight into the response.
pub enum Answer {
    Body(StatusCode, RpcResponse),
    Accepted,
}

impl Answer {
    pub(super) fn ok(response: RpcResponse) -> Self {
        Self::Body(StatusCode::OK, response)
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Body(status, _) => *status,
            Self::Accepted => StatusCode::ACCEPTED,
        }
    }

    pub fn body(&self) -> Option<&RpcResponse> {
        match self {
            Self::Body(_, body) => Some(body),
            Self::Accepted => None,
        }
    }
}

impl IntoResponse for Answer {
    fn into_response(self) -> Response {
        match self {
            Self::Accepted => StatusCode::ACCEPTED.into_response(),
            Self::Body(status, body) => {
                let mut response = (status, serde_json::to_string(&body).unwrap_or_default()).into_response();
                response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
                response
            }
        }
    }
}
