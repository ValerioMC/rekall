//! What a guard hook asks Rekall before it decides, served by `rekall_console::guard`.

mod controller;
mod request;
mod response;
mod routes;

pub use response::guard_context_response::GuardContextResponse;
pub use response::running_step_response::RunningStepResponse;
pub(crate) use routes::routes;

use request::guard_context_query::GuardContextQuery;
