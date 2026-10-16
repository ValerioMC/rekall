use axum::http::StatusCode;
use rekall_common::{Id, RekallError};

use crate::error::{ApiError, ApiResult};

/// A `UUID` path variable or query parameter named `name`; anything else is the caller's 400.
pub fn uuid(name: &str, raw: &str) -> ApiResult<Id> {
    raw.parse()
        .map_err(|_| ApiError::Domain(RekallError::illegal(format!("Parameter '{name}' is not a UUID: {raw}"))))
}

/// An optional `UUID` query parameter; an empty value reads as absent.
pub fn optional_uuid(name: &str, raw: Option<&str>) -> ApiResult<Option<Id>> {
    match raw {
        None => Ok(None),
        Some("") => Ok(None),
        Some(value) => uuid(name, value).map(Some),
    }
}

/// A required query parameter that was not sent.
pub fn required<T>(value: Option<T>) -> ApiResult<T> {
    value.ok_or(ApiError::Framework(StatusCode::BAD_REQUEST))
}

/// Every path variable of the matched route read as a `UUID` before the body is, so a bad id is
/// reported even when the body is bad too. Layered with `route_layer`, after routing.
pub async fn uuid_path_params(
    params: axum::extract::RawPathParams,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    for (name, value) in &params {
        if let Err(refused) = uuid(&camel_case(name), value) {
            return axum::response::IntoResponse::into_response(refused);
        }
    }
    next.run(request).await
}

/// `task_id` -> `taskId`: the parameter name the console's requests use.
fn camel_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}
