use axum::http::StatusCode;
use rekall_common::{Id, RekallError};

use crate::error::{ApiError, ApiResult};

/// A `UUID` path variable or query parameter named `name`.
pub fn uuid(name: &str, raw: &str) -> ApiResult<Id> {
    raw.parse().map_err(|_| {
        ApiError::Domain(RekallError::internal(
            "MethodArgumentTypeMismatchException",
            format!(
                "Method parameter '{name}': Failed to convert value of type 'java.lang.String' to required type \
                 'java.util.UUID'; Invalid UUID string: {raw}"
            ),
        ))
    })
}

/// An optional `UUID` query parameter; an empty value reads as absent, as Spring converted it.
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

/// Every path variable of the matched route read as a `UUID` before anything else of the
/// request is: Spring converted `@PathVariable UUID` arguments ahead of the `@RequestBody`, so a
/// bad id is its 500 even when the body is bad too. Layered with `route_layer`, after routing.
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

/// `task_id` -> `taskId`: the Java parameter a route segment was bound to.
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
