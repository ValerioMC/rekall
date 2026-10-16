//! `SettingsController`: the database folders, as the setup screen and the Settings panel manage
//! them. Adding or switching one rewrites `config.json` and restarts onto it.

mod add_request;
mod add_response;
mod check_query;
mod check_response;
mod database_view;
mod rename_request;
mod settings_controller;
mod settings_state;
mod status_response;

pub use database_view::DatabaseView;
pub use settings_controller::routes;
pub use settings_state::SettingsState;
pub use status_response::StatusResponse;

use add_request::AddRequest;
use add_response::AddResponse;
use check_query::CheckQuery;
use check_response::CheckResponse;
use rename_request::RenameRequest;
