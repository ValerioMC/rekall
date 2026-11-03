//! The quick task: a task started from one line of text, with its session opened at once. Its
//! wrapup, once Claude writes it, accepts it without a review.

mod command;
mod rule;
mod service;
mod view;

pub use command::quick_task_command::QuickTaskCommand;
pub use service::quick_task_service::QuickTaskService;
pub use view::quick_task_view::QuickTaskView;
