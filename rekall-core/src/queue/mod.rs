//! The run queue: tasks queued to run one after another, its settings and the moves of each item.
//! The worker that opens a terminal for each is reached through `QueueRunner`.

mod queue_runner;
mod run_queue_frame;
mod run_queue_item_view;
mod run_queue_service;
mod run_queue_view;
mod snapshot;

pub use queue_runner::QueueRunner;
pub use run_queue_frame::RunQueueFrame;
pub use run_queue_item_view::RunQueueItemView;
pub use run_queue_service::{RunQueueService, CEILING_MAX, CEILING_MIN};
pub use run_queue_view::RunQueueView;
pub use snapshot::Snapshot;
