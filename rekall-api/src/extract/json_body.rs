use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

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
