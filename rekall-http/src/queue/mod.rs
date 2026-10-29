//! The routes of the run queue, served by `rekall_console::queue`.

mod controller;
mod request;
mod routes;

use request::add_request::AddRequest;
use request::move_request::MoveRequest;
use request::settings_request::SettingsRequest;
use request::start_request::StartRequest;

pub(crate) use routes::routes;
