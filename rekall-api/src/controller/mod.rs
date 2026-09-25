//! One router per Java controller, under the same paths.

mod catalog;
mod commit_reference;
mod company;
mod context_size;
mod directory_listing;
mod document;
mod export;
mod search;
mod step_event;
mod tag;
mod task_revision;
mod task_step;
mod time_entry;
mod wrapup;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Router;
use serde::Serialize;

use crate::ApiState;

pub fn routes() -> Router<ApiState> {
    Router::new()
        .merge(catalog::routes())
        .merge(commit_reference::routes())
        .merge(company::routes())
        .merge(context_size::routes())
        .merge(directory_listing::routes())
        .merge(document::routes())
        .merge(export::routes())
        .merge(search::routes())
        .merge(step_event::routes())
        .merge(tag::routes())
        .merge(task_revision::routes())
        .merge(task_step::routes())
        .merge(time_entry::routes())
        .merge(wrapup::routes())
}

/// `@ResponseStatus(HttpStatus.CREATED)` on a JSON answer.
pub fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, axum::Json(body)).into_response()
}

/// `@ResponseStatus(HttpStatus.NO_CONTENT)` on a void method.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}
