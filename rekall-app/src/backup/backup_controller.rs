use std::io::Write;

use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_web::error::{framework, ApiResult};
use rekall_common::{InternalFailure, RekallError};

use super::{BackupFile, BackupReason, BackupStatus, DatabaseBackupService, DatabaseRestoreService};

use super::BackupState;
use super::RestoreStarted;

/// The largest upload accepted: a backup is the whole database zipped.
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;

pub fn routes(backups: DatabaseBackupService, restorer: DatabaseRestoreService) -> Router {
    let state = BackupState { backups, restorer };
    Router::new()
        .route("/api/backups", get(status).post(back_up_now))
        .route("/api/backups/restore", post(restore_upload))
        .route("/api/backups/{name}", get(download))
        .route("/api/backups/{name}/restore", post(restore))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .with_state(state)
}

async fn status(State(state): State<BackupState>) -> ApiResult<Json<BackupStatus>> {
    Ok(Json(state.backups.status()?))
}

async fn back_up_now(State(state): State<BackupState>) -> ApiResult<Json<BackupFile>> {
    Ok(Json(state.backups.back_up(BackupReason::Manual).await?))
}

async fn download(State(state): State<BackupState>, Path(name): Path<String>) -> ApiResult<Response> {
    let file = state.backups.resolve(&name)?;
    let bytes = tokio::fs::read(&file)
        .await
        .map_err(|_| RekallError::internal(InternalFailure::Io, format!("Could not read {}", file.display())))?;
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

async fn restore(State(state): State<BackupState>, Path(name): Path<String>) -> ApiResult<Json<RestoreStarted>> {
    let previous_state = state.restorer.restore(&name).await?;
    Ok(Json(RestoreStarted { restarting: true, previous_state }))
}

/// The upload is the multipart part named `file`; a request that is not multipart is a 400.
async fn restore_upload(State(state): State<BackupState>, multipart: Result<Multipart, axum::extract::multipart::MultipartRejection>) -> ApiResult<Response> {
    let Ok(mut multipart) = multipart else {
        return Ok(framework(StatusCode::BAD_REQUEST));
    };
    let unreadable = || RekallError::internal(InternalFailure::Io, "Could not read the uploaded file");
    let mut upload: Option<(tempfile::NamedTempFile, u64)> = None;
    while let Some(mut field) = multipart.next_field().await.map_err(|_| unreadable())? {
        if field.name() != Some("file") {
            continue;
        }
        let mut temp = tempfile::NamedTempFile::new().map_err(|_| unreadable())?;
        let mut size = 0u64;
        while let Some(chunk) = field.chunk().await.map_err(|_| unreadable())? {
            size += chunk.len() as u64;
            temp.write_all(&chunk).map_err(|_| unreadable())?;
        }
        temp.flush().map_err(|_| unreadable())?;
        upload = Some((temp, size));
        break;
    }
    let Some((temp, size)) = upload else {
        return Ok(framework(StatusCode::BAD_REQUEST));
    };
    if size == 0 {
        return Err(RekallError::illegal("The uploaded file is empty. Nothing was restored.").into());
    }
    let previous_state = state.restorer.restore_upload(temp.path()).await?;
    Ok(Json(RestoreStarted { restarting: true, previous_state }).into_response())
}
