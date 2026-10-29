//! The worker that runs the queue: tasks one after another, each in a `claude` terminal of its own,
//! held at a usage ceiling. The queue itself is `rekall_core::queue`.

mod add_request;
mod move_request;
mod queue_controller;
mod run_queue_runner;
mod run_queue_worker;
mod settings_request;
mod start_request;
pub mod usage_ceiling;

pub use queue_controller::routes;
pub use run_queue_runner::RunQueueRunner;

use add_request::AddRequest;
use move_request::MoveRequest;
use settings_request::SettingsRequest;
use start_request::StartRequest;
