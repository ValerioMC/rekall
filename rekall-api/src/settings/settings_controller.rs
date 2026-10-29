use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use rekall_web::error::{framework, ApiResult};
use rekall_web::extract::JsonBody;

use super::{AddRequest, AddResponse, CheckQuery, CheckResponse, DatabaseResponse, RenameRequest, StatusResponse};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/settings/databases", get(status).post(add))
        .route("/api/settings/databases/check", get(check))
        .route("/api/settings/databases/{id}/activate", post(activate))
        .route("/api/settings/databases/{id}", patch(rename).delete(forget))
}

async fn status(State(state): State<ApiState>) -> ApiResult<Json<StatusResponse>> {
    Ok(Json(state.database_folders.status()?.into()))
}

/// `path` is a required query parameter; without it the request is a 400.
async fn check(State(state): State<ApiState>, Query(query): Query<CheckQuery>) -> Response {
    let Some(path) = query.path else {
        return framework(StatusCode::BAD_REQUEST);
    };
    Json(CheckResponse::from(state.database_folders.check(&path))).into_response()
}

async fn add(State(state): State<ApiState>, JsonBody(request): JsonBody<AddRequest>) -> ApiResult<Response> {
    let added = state.database_folders.add(request.into())?;
    Ok((StatusCode::CREATED, Json(AddResponse::from(added))).into_response())
}

async fn activate(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<Json<DatabaseResponse>> {
    Ok(Json(state.database_folders.activate(&id)?.into()))
}

async fn rename(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<RenameRequest>,
) -> ApiResult<Json<DatabaseResponse>> {
    Ok(Json(state.database_folders.rename(&id, request.label.as_deref())?.into()))
}

async fn forget(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    state.database_folders.forget(&id)?;
    Ok(StatusCode::NO_CONTENT)
}
