//! The one write path for a task's wrapup. A write replaces the text outright and a delete removes
//! it: no earlier version is kept.

mod event;
mod model;
mod service;
mod view;

pub use service::wrapup_service::WrapupService;
pub use event::wrapup_stream_event::WrapupStreamEvent;
pub use view::wrapup_view::WrapupView;
pub use model::written::Written;
pub use rekall_store::WrapupAuthor;
