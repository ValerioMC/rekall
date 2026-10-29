//! A task's wrapup.

mod controller;
mod request;
mod routes;

pub use request::wrapup_request::WrapupRequest;
pub(crate) use routes::routes;
