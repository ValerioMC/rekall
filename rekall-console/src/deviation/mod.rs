//! What a task did differently from its plan, as the console reads it. Read-only here: a session
//! writes a deviation through `rekall-core`, and the console only shows it.

mod service;
mod view;

pub use service::task_deviation_service::TaskDeviationService;
pub use view::task_deviation_view::TaskDeviationView;
