//! Tags.

mod tag_controller;
mod tag_request;
mod tag_response;
mod routes;

pub use tag_request::TagRequest;
pub use tag_response::TagResponse;
pub(crate) use routes::routes;
