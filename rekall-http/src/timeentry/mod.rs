//! Time logged against a task.

mod time_entry_controller;
mod time_entry_edit_request;
mod routes;

pub use time_entry_edit_request::TimeEntryEditRequest;
pub(crate) use routes::routes;
