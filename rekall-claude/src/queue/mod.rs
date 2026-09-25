//! The run queue: tasks queued to run one after another, each in a `claude` terminal of its own,
//! held at a usage ceiling.

mod controller;
mod runner;
mod service;
pub mod usage_ceiling;
mod view;

pub use controller::routes;
pub use runner::RunQueueRunner;
pub use service::{RunQueueService, Snapshot, CEILING_MAX, CEILING_MIN};
pub use view::{RunQueueItemView, RunQueueView};
