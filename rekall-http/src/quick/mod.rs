//! Starting a quick task from one line of text.

mod controller;
mod request;
mod response;
mod routes;

pub use request::quick_task_request::QuickTaskRequest;
pub use response::quick_task_response::QuickTaskResponse;
pub(crate) use routes::routes;
