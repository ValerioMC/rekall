//! The Server-Sent Events feed the console listens to.

mod controller;
mod support;
mod routes;

pub use support::step_event_stream::StepEventStream;
pub(crate) use routes::routes;
