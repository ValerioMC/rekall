//! Time logged against a task.

mod controller;
mod request;
mod routes;

pub use request::time_entry_edit_request::TimeEntryEditRequest;
pub(crate) use routes::routes;
