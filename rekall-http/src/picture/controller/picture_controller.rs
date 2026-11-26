use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use rekall_console::picture::{too_large, PictureImportCommand, PictureUploadCommand, MAX_PICTURE_BYTES};

use crate::picture::request::picture_import_request::PictureImportRequest;
use crate::picture::request::picture_owner_request::PictureOwnerRequest;
use crate::picture::request::picture_thumbnail_query::PictureThumbnailQuery;
use crate::picture::request::picture_upload_query::PictureUploadQuery;
use crate::picture::response::imported_picture_response::ImportedPictureResponse;
use crate::picture::response::picture_file_response::PictureFileResponse;
use crate::shared::created;
use crate::shared::error::{ApiError, ApiResult};
use crate::shared::extract::{present, uuid, JsonBody, Validator};
use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/api/images", post(import))
        .route("/api/image-uploads", post(upload).layer(DefaultBodyLimit::max(MAX_PICTURE_BYTES as usize)))
        .route("/api/images/{*path}", get(show))
        .route("/api/image-thumbnail", get(thumbnail))
}

async fn import(State(state): State<ApiState>, JsonBody(request): JsonBody<PictureImportRequest>) -> ApiResult<Response> {
    Validator::new().not_blank("sourcePath", request.source_path.as_deref()).not_null("owner", &request.owner).finish()?;
    let command = PictureImportCommand {
        source_path: present("sourcePath", request.source_path)?,
        owner: present("owner", request.owner)?.into(),
    };
    Ok(created(ImportedPictureResponse::from(state.pictures.import(command).await?)))
}

/// The request body is the picture file itself.
async fn upload(
    State(state): State<ApiState>,
    Query(query): Query<PictureUploadQuery>,
    body: Result<Bytes, BytesRejection>,
) -> ApiResult<Response> {
    Validator::new().not_null("ownerKind", &query.owner_kind).not_blank("ownerId", query.owner_id.as_deref()).finish()?;
    let owner = PictureOwnerRequest {
        kind: present("ownerKind", query.owner_kind)?,
        id: uuid("ownerId", &present("ownerId", query.owner_id)?)?,
    };
    let command = PictureUploadCommand { name: query.name, bytes: body.map_err(refused_body)?.into(), owner: owner.into() };
    Ok(created(ImportedPictureResponse::from(state.pictures.upload(command).await?)))
}

/// A body past the route's limit is refused in the words a picked file that size is.
fn refused_body(rejection: BytesRejection) -> ApiError {
    if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return too_large().into();
    }
    ApiError::Unreadable(rejection.body_text())
}

async fn show(State(state): State<ApiState>, Path(path): Path<String>) -> ApiResult<PictureFileResponse> {
    Ok(state.pictures.read(&path).await?.into())
}

async fn thumbnail(State(state): State<ApiState>, Query(query): Query<PictureThumbnailQuery>) -> ApiResult<PictureFileResponse> {
    Validator::new().not_blank("path", query.path.as_deref()).finish()?;
    let path = present("path", query.path)?;
    Ok(PictureFileResponse::revalidated(state.pictures.thumbnail(&path).await?))
}
