use axum::extract::multipart::{Field, MultipartRejection};
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::StreamExt;
use rekall_core::backup::{BackupFile, BackupStatus, UploadChunks};
use rekall_kernel::{InternalFailure, RekallError};
use crate::shared::error::{framework, ApiResult};

use crate::backup::RestoreStarted;
use crate::ApiState;

/// The largest upload accepted: a backup is the whole database zipped.
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/backups", get(status).post(back_up_now))
        .route("/api/backups/restore", post(restore_upload))
        .route("/api/backups/{name}", get(download))
        .route("/api/backups/{name}/restore", post(restore))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
}

async fn status(State(state): State<ApiState>) -> ApiResult<Json<BackupStatus>> {
    Ok(Json(state.backups.status()?))
}

async fn back_up_now(State(state): State<ApiState>) -> ApiResult<Json<BackupFile>> {
    Ok(Json(state.backups.back_up_now().await?))
}

async fn download(State(state): State<ApiState>, Path(name): Path<String>) -> ApiResult<Response> {
    let mut response = state.backups.download(&name).await?.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

async fn restore(State(state): State<ApiState>, Path(name): Path<String>) -> ApiResult<Json<RestoreStarted>> {
    let previous_state = state.backups.restore(&name).await?;
    Ok(Json(RestoreStarted { restarting: true, previous_state }))
}

/// The upload is the multipart part named `file`; a request that is not multipart, or has no such part, is a 400.
async fn restore_upload(State(state): State<ApiState>, multipart: Result<Multipart, MultipartRejection>) -> ApiResult<Response> {
    let Ok(mut multipart) = multipart else {
        return Ok(framework(StatusCode::BAD_REQUEST));
    };
    while let Some(field) = multipart.next_field().await.map_err(|_| unreadable())? {
        if field.name() != Some("file") {
            continue;
        }
        let previous_state = state.backups.restore_upload(chunks_of(field)).await?;
        return Ok(Json(RestoreStarted { restarting: true, previous_state }).into_response());
    }
    Ok(framework(StatusCode::BAD_REQUEST))
}

fn chunks_of(field: Field<'_>) -> UploadChunks<'_> {
    field.map(|chunk| chunk.map(|bytes| bytes.to_vec()).map_err(|_| unreadable())).boxed()
}

fn unreadable() -> RekallError {
    RekallError::internal(InternalFailure::Io, "Could not read the uploaded file")
}
