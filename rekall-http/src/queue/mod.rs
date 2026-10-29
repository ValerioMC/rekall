//! The routes of the run queue, served by `rekall_console::queue`.

mod add_request;
mod move_request;
mod queue_controller;
mod routes;
mod settings_request;
mod start_request;

use add_request::AddRequest;
use move_request::MoveRequest;
use settings_request::SettingsRequest;
use start_request::StartRequest;

pub(crate) use routes::routes;
