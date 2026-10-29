//! The run queue: tasks queued to run one after another, its settings and the moves of each item.
//! The worker that opens a terminal for each is reached through `QueueRunner`.

mod model;
mod port;
mod service;
mod view;

pub use port::queue_runner::QueueRunner;
pub use model::run_queue_frame::RunQueueFrame;
pub use view::run_queue_item_view::RunQueueItemView;
pub use service::run_queue_service::{RunQueueService, CEILING_MAX, CEILING_MIN};
pub use view::run_queue_view::RunQueueView;
pub use model::snapshot::Snapshot;
