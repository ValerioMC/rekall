//! The one write path for a task's wrapup. A write replaces the text outright and a delete removes
//! it: no earlier version is kept.

mod wrapup_service;
mod wrapup_stream_event;
mod wrapup_view;
mod written;

pub use wrapup_service::WrapupService;
pub use wrapup_stream_event::WrapupStreamEvent;
pub use wrapup_view::WrapupView;
pub use written::Written;
