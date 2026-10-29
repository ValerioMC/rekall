//! Tags.

mod controller;
mod request;
mod response;
mod routes;

pub use request::tag_request::TagRequest;
pub use response::tag_response::TagResponse;
pub(crate) use routes::routes;
