//! A task's decisions: read for the Decisions tab, and edited, added or removed by hand. A session
//! writes through `rekall-mcp`; the console learns of it by the `decision` frame of the event stream.

mod controller;
mod request;
mod response;
mod routes;

pub use request::add_decision_request::AddDecisionRequest;
pub use request::edit_decision_request::EditDecisionRequest;
pub use response::decision_response::DecisionResponse;
pub use response::task_decisions_response::TaskDecisionsResponse;
pub(crate) use routes::routes;
