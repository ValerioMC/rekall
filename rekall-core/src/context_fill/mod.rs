//! The last context fill of a task and of each of its steps, kept apart from the task and step
//! rows so a write per turn moves neither of their `updated_at`.

mod event;
mod service;
mod view;

pub use event::context_fill_stream_event::ContextFillStreamEvent;
pub use service::context_fill_service::ContextFillService;
pub use view::context_fill_view::ContextFillView;
