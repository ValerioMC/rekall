use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

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
