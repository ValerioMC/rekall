//! A task's thread under its description: the person adds feedback, edits and deletes it while it
//! is pending, and discards a revision. A session answers through `rekall-mcp`.

mod controller;
mod request;
mod response;
mod routes;

pub use request::add_feedback_request::AddFeedbackRequest;
pub use request::edit_feedback_request::EditFeedbackRequest;
pub use response::feedback_response::FeedbackResponse;
pub(crate) use routes::routes;
