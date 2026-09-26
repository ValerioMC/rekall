//! The one write path for a task's wrapup. Whatever a write replaces, and whatever a delete
//! removes, is handed to the revision service first, so the console's history can bring it back;
//! that includes an edit made by hand, which a session's write would otherwise erase.

mod wrapup_service;
mod wrapup_stream_event;
mod wrapup_view;
mod written;

pub use wrapup_service::WrapupService;
pub use wrapup_stream_event::WrapupStreamEvent;
pub use wrapup_view::WrapupView;
pub use written::Written;
