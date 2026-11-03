//! What a person decides about the order of tasks: which task another one waits for. Console-only,
//! since a session that could write its own dependencies could unblock itself.

mod command;
mod service;
mod view;

pub use command::task_dependency_command::TaskDependencyCommand;
pub use service::task_dependency_service::TaskDependencyService;
pub use view::task_dependency_view::TaskDependencyView;
