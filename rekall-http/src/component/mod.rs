//! A project's components.

mod controller;
mod request;
mod routes;

pub use request::project_component_order_request::ProjectComponentOrderRequest;
pub use request::project_component_request::ProjectComponentRequest;
pub(crate) use routes::routes;
