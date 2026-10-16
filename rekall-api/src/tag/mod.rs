//! Tags.

mod tag_controller;
mod tag_request;
mod tag_response;
mod tag_service;
mod routes;

pub use tag_request::TagRequest;
pub use tag_response::TagResponse;
pub use tag_service::TagService;
pub(crate) use routes::routes;
