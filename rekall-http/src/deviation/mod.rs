//! What a task did differently from its plan: read-only, since only a session writes a deviation.

mod controller;
mod response;
mod routes;

pub use response::task_deviation_response::TaskDeviationResponse;
pub(crate) use routes::routes;
