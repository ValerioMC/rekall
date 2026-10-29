//! The worker that runs the queue: tasks one after another, each in a `claude` terminal of its own,
//! held at a usage ceiling. The queue itself is `rekall_core::queue`.

mod run_queue_runner;
mod run_queue_worker;
pub mod usage_ceiling;

pub use run_queue_runner::RunQueueRunner;
