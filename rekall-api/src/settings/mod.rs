//! The database folders, as the setup screen and the Settings panel manage them, served by
//! `rekall_console::settings`. Adding or switching one restarts the application onto it.

mod add_request;
mod add_response;
mod check_query;
mod check_response;
mod database_response;
mod rename_request;
mod routes;
mod settings_controller;
mod status_response;

pub use database_response::DatabaseResponse;
pub use status_response::StatusResponse;
pub(crate) use routes::routes;

use add_request::AddRequest;
use add_response::AddResponse;
use check_query::CheckQuery;
use check_response::CheckResponse;
use rename_request::RenameRequest;
