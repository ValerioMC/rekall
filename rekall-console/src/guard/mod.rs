//! What a guard hook needs to know about Rekall before it decides: the project of a folder, whether
//! it commits for the session, and which step is running.

mod command;
mod service;
mod view;

pub use command::guard_context_command::GuardContextCommand;
pub use service::guard_context_service::GuardContextService;
pub use view::guard_context_view::GuardContextView;
pub use view::running_step_view::RunningStepView;
