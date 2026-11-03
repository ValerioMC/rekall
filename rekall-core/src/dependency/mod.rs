//! A task that waits for another one of its project to be accepted. The console writes the
//! dependencies; here they are read as what a task still waits for, which the run queue and the
//! context both show.

mod model;
mod rule;
mod service;

pub use model::task_wait::{TaskWait, AWAITING_REVIEW_TEXT};
pub use rule::dependency_cycle::closes_a_cycle;
pub use service::task_wait_service::TaskWaitService;
