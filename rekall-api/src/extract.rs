//! Request reading with Spring MVC's outcomes: a body Jackson cannot read is a 400 with the
//! parser's reason, a path variable that is not a UUID is the 500 the catch-all handler gave a
//! `MethodArgumentTypeMismatchException`, and a missing required query parameter is Spring's own 400.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use rekall_common::{jstr, Id, RekallError};
use serde::de::DeserializeOwned;

use crate::error::{ApiError, ApiResult};

/// `@RequestBody`: required, JSON.
pub struct JsonBody<T>(pub T);

impl<S, T> FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::Unreadable(e.body_text()))?;
        if bytes.iter().all(u8::is_ascii_whitespace) {
            return Err(ApiError::Unreadable("Required request body is missing".into()));
        }
        serde_json::from_slice(&bytes).map(JsonBody).map_err(|e| ApiError::Unreadable(e.to_string()))
    }
}

/// `@RequestBody(required = false)`: an absent body reads as null.
pub struct OptionalJsonBody<T>(pub Option<T>);

impl<S, T> FromRequest<S> for OptionalJsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|e| ApiError::Unreadable(e.body_text()))?;
        if bytes.iter().all(u8::is_ascii_whitespace) {
            return Ok(OptionalJsonBody(None));
        }
        let value: Option<T> = serde_json::from_slice(&bytes).map_err(|e| ApiError::Unreadable(e.to_string()))?;
        Ok(OptionalJsonBody(value))
    }
}

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
        Some(value) if value.is_empty() => Ok(None),
        Some(value) => uuid(name, value).map(Some),
    }
}

/// A required query parameter that was not sent.
pub fn required<T>(value: Option<T>) -> ApiResult<T> {
    value.ok_or(ApiError::Framework(StatusCode::BAD_REQUEST))
}

/// The `@Valid` constraints of a request body, checked in declaration order.
#[derive(Default)]
pub struct Validator {
    errors: Vec<(String, String)>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    /// `@NotBlank`.
    pub fn not_blank(mut self, field: &str, value: Option<&str>) -> Self {
        if jstr::is_null_or_blank(value) {
            self.errors.push((field.into(), "must not be blank".into()));
        }
        self
    }

    /// `@NotNull`.
    pub fn not_null<T>(mut self, field: &str, value: &Option<T>) -> Self {
        if value.is_none() {
            self.errors.push((field.into(), "must not be null".into()));
        }
        self
    }

    pub fn finish(self) -> ApiResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::Validation(self.errors))
        }
    }
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
