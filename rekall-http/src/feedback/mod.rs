//! A task's revisions under its description: the person points a waiting one at the work or the
//! plan, starts a pass over the waiting ones, or discards one. A session answers through `rekall-mcp`.

mod controller;
mod request;
mod response;
mod routes;

pub use request::retarget_feedback_request::RetargetFeedbackRequest;
pub use response::feedback_response::FeedbackResponse;
pub(crate) use routes::routes;
