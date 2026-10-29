//! The database folders, as the setup screen and the Settings panel manage them, served by
//! `rekall_console::settings`. Adding or switching one restarts the application onto it.

mod controller;
mod request;
mod response;
mod routes;

pub use response::database_response::DatabaseResponse;
pub use response::status_response::StatusResponse;
pub(crate) use routes::routes;

use request::add_request::AddRequest;
use response::add_response::AddResponse;
use request::check_query::CheckQuery;
use response::check_response::CheckResponse;
use request::rename_request::RenameRequest;
