//! A task's thread under its description. The only write a session has is `reply`: it answers
//! pending entries and cannot create, edit or delete one. The context carries the thread in a
//! `<thread>` block, pending entries in full and applied ones as one line.

mod event;
mod model;
mod service;
mod view;

pub use event::feedback_stream_event::FeedbackStreamEvent;
pub use model::replied::Replied;
pub use service::feedback_service::FeedbackService;
pub use view::feedback_view::FeedbackView;
