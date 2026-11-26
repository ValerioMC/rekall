use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use rekall_console::picture::PictureImportCommand;

use crate::picture::request::picture_import_request::PictureImportRequest;
use crate::picture::response::imported_picture_response::ImportedPictureResponse;
use crate::picture::response::picture_file_response::PictureFileResponse;
use crate::shared::created;
use crate::shared::error::ApiResult;
use crate::shared::extract::{present, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/images", post(import))
        .route("/api/images/{*path}", get(show))
}

async fn import(State(state): State<ApiState>, JsonBody(request): JsonBody<PictureImportRequest>) -> ApiResult<Response> {
    Validator::new().not_blank("sourcePath", request.source_path.as_deref()).not_null("owner", &request.owner).finish()?;
    let command = PictureImportCommand {
        source_path: present("sourcePath", request.source_path)?,
        owner: present("owner", request.owner)?.into(),
    };
    Ok(created(ImportedPictureResponse::from(state.pictures.import(command).await?)))
}

async fn show(State(state): State<ApiState>, Path(path): Path<String>) -> ApiResult<PictureFileResponse> {
    Ok(state.pictures.read(&path).await?.into())
}
