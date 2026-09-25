use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::error::ApiResult;
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new().route("/api/export", get(download))
}

async fn download(State(state): State<ApiState>) -> ApiResult<Response> {
    let archive = state.export.archive().await?;
    let disposition = format!("attachment; filename=\"{}\"", state.export.file_name_for_today());
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
            (header::CONTENT_LENGTH, archive.len().to_string()),
        ],
        archive,
    )
        .into_response())
}
