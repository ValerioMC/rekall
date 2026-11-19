//! A task's thread under its description. The only writes a session has are `reply`, which answers
//! the open entries (waiting or in a pass) and cannot create, edit or delete one, and the release of
//! a pass whose session ended. The context carries the thread in a `<thread>` block, open entries in
//! full and applied ones as one line.

mod event;
mod model;
mod service;
mod view;

pub use event::feedback_stream_event::FeedbackStreamEvent;
pub use model::replied::Replied;
pub use service::feedback_service::FeedbackService;
pub use view::feedback_view::FeedbackView;
