use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rekall_console::export::ProjectExportCommand;

use crate::export::request::project_export_clear_query::ProjectExportClearQuery;
use crate::export::request::project_export_request::ProjectExportRequest;
use crate::export::response::project_export_cleared_response::ProjectExportClearedResponse;
use crate::export::response::project_export_response::ProjectExportResponse;
use crate::shared::error::ApiResult;
use crate::shared::extract::{uuid, JsonBody};
use crate::shared::{created, no_content};
use crate::ApiState;

/// The request carries a picture per diagram, base64 in JSON.
const MAX_REQUEST_BYTES: usize = 64 * 1024 * 1024;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/projects/{id}/exports", post(export))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .route("/api/exports", get(list).delete(clear))
        .route("/api/exports/{id}/file", get(download))
        .route("/api/exports/{id}", delete(delete_one))
}

async fn export(State(state): State<ApiState>, Path(id): Path<String>, JsonBody(request): JsonBody<ProjectExportRequest>) -> ApiResult<Response> {
    let project_id = uuid("id", &id)?;
    let command = ProjectExportCommand::try_from(request)?;
    Ok(created(ProjectExportResponse::from(state.exports.export(project_id, command).await?)))
}

async fn list(State(state): State<ApiState>) -> ApiResult<Json<Vec<ProjectExportResponse>>> {
    Ok(Json(state.exports.list().await?.into_iter().map(ProjectExportResponse::from).collect()))
}

async fn download(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    let file = state.exports.file(uuid("id", &id)?).await?;
    let mut response = file.bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(file.media_type));
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.file_name)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

async fn delete_one(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Response> {
    state.exports.delete(uuid("id", &id)?).await?;
    Ok(no_content())
}

async fn clear(State(state): State<ApiState>, Query(query): Query<ProjectExportClearQuery>) -> ApiResult<Json<ProjectExportClearedResponse>> {
    let deleted = state.exports.clear(query.try_into()?).await?;
    Ok(Json(ProjectExportClearedResponse { deleted }))
}
